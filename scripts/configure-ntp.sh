#!/bin/bash
# NTPsec Runtime Configuration Generator
# Generates NTP configuration based on detected hardware and environment variables

set -e

CONFIG_DIR="/etc/ntpsec/ntp.d"
TEMPLATE="/etc/ntpsec/ntp.conf.template"
OUTPUT="/etc/ntpsec/ntp.conf"

# Environment variables with defaults
NTP_SERVERS="${NTP_SERVERS:-time.cloudflare.com,time.google.com,time.nist.gov}"
ENABLE_PPS="${ENABLE_PPS:-auto}"
ENABLE_GPS="${ENABLE_GPS:-auto}"
PPS_GPIO="${PPS_GPIO:-4}"
GPS_DEVICE="${GPS_DEVICE:-/dev/ttyAMA0}"
GPS_BAUD="${GPS_BAUD:-9600}"
LOCAL_STRATUM="${LOCAL_STRATUM:-10}"
NTP_STATS_ENABLED="${NTP_STATS_ENABLED:-yes}"
NTP_HARDWARE_TIMESTAMPING="${NTP_HARDWARE_TIMESTAMPING:-auto}"

log() {
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"
}

# Create config directory
mkdir -p "$CONFIG_DIR"

log "Configuring NTPsec..."
log "NTP Servers: $NTP_SERVERS"
log "PPS Enabled: $ENABLE_PPS"
log "GPS Enabled: $ENABLE_GPS"

# Detect hardware if auto mode
if [ "$ENABLE_PPS" = "auto" ] || [ "$ENABLE_GPS" = "auto" ]; then
    log "Detecting hardware..."
    HARDWARE_DETECT=$(/usr/local/bin/detect-hardware.sh)
    
    # Parse detection results
    if echo "$HARDWARE_DETECT" | grep -q "found:pps:"; then
        HAS_PPS=true
        log "Detected PPS device"
    fi
    
    if echo "$HARDWARE_DETECT" | grep -q "found:gps:"; then
        HAS_GPS=true
        GPS_DETECTED_DEVICE=$(echo "$HARDWARE_DETECT" | grep "found:gps:" | cut -d: -f3)
        log "Detected GPS on $GPS_DETECTED_DEVICE"
    fi
fi

# Configure hardware clocks
log "Generating hardware configuration..."
cat > "$CONFIG_DIR/20-hardware.conf" <<EOF
# Hardware Reference Clocks
# Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
# Configuration: PPS=$ENABLE_PPS, GPS=$ENABLE_GPS

EOF

# Configure PPS (Pulse Per Second)
if [ "$ENABLE_PPS" = "yes" ] || [ "$ENABLE_PPS" = "auto" -a "$HAS_PPS" = "true" ]; then
    log "Configuring PPS reference clock..."
    
    # Load PPS kernel module if not loaded
    if ! lsmod | grep -q pps_gpio; then
        log "Loading pps-gpio module..."
        modprobe pps-gpio gpio_pin="$PPS_GPIO" 2>/dev/null || log "Warning: Could not load pps-gpio module"
    fi
    
    # Enable PPS device
    if [ -c /dev/pps0 ]; then
        echo 1 > /sys/class/pps/pps0/enable 2>/dev/null || true
    fi
    
    cat >> "$CONFIG_DIR/20-hardware.conf" <<EOF
# PPS Reference Clock (GPIO $PPS_GPIO)
# Provides microsecond-accurate timing
server 127.127.22.0 minpoll 4 maxpoll 4 true
fudge 127.127.22.0 refid PPS flag3 1 flag4 1

EOF
    log "PPS configured successfully"
fi

# Configure GPS
if [ "$ENABLE_GPS" = "yes" ] || [ "$ENABLE_GPS" = "auto" -a "$HAS_GPS" = "true" ]; then
    GPS_DEVICE=${GPS_DETECTED_DEVICE:-$GPS_DEVICE}
    log "Configuring GPS reference clock on $GPS_DEVICE..."
    
    # Configure GPS serial port
    if [ -c "$GPS_DEVICE" ]; then
        stty -F "$GPS_DEVICE" "$GPS_BAUD" cs8 -cstopb -parenb 2>/dev/null || log "Warning: Could not configure GPS serial port"
    fi
    
    cat >> "$CONFIG_DIR/20-hardware.conf" <<EOF
# GPS/NMEA Reference Clock
# Provides coarse time synchronization
server 127.127.28.0 mode 17 minpoll 4 maxpoll 4 iburst prefer
fudge 127.127.28.0 refid GPS time1 0.0 flag1 1

EOF
    log "GPS configured successfully"
fi

# If no hardware clocks, add note
if [ ! -s "$CONFIG_DIR/20-hardware.conf" ] || [ "$(wc -l < "$CONFIG_DIR/20-hardware.conf")" -lt 5 ]; then
    cat >> "$CONFIG_DIR/20-hardware.conf" <<EOF
# No hardware reference clocks configured
# Using network time servers only

EOF
fi

# Configure network servers
log "Configuring network time servers..."
cat > "$CONFIG_DIR/10-servers.conf" <<EOF
# Network Time Servers
# Configured servers: $NTP_SERVERS

EOF

IFS=',' read -ra SERVERS <<< "$NTP_SERVERS"
for server in "${SERVERS[@]}"; do
    server=$(echo "$server" | xargs)  # Trim whitespace
    echo "server $server iburst" >> "$CONFIG_DIR/10-servers.conf"
    log "Added server: $server"
done

# Configure statistics if enabled
if [ "$NTP_STATS_ENABLED" = "yes" ]; then
    cat > "$CONFIG_DIR/30-stats.conf" <<EOF
# Statistics Configuration
statistics loopstats peerstats clockstats
filegen loopstats file loopstats type day enable
filegen peerstats file peerstats type day enable
filegen clockstats file clockstats type day enable

EOF
fi

# Generate final configuration
log "Generating final NTP configuration..."
envsubst < "$TEMPLATE" > "$OUTPUT"

# Set permissions
chmod 644 "$OUTPUT"
chmod 644 "$CONFIG_DIR"/*.conf

log "NTP configuration generated successfully"
log "Configuration file: $OUTPUT"

# Validate configuration
if command -v ntpd >/dev/null 2>&1; then
    if ! ntpd -n -c "$OUTPUT" -d 2>&1 | grep -q "syntax error"; then
        log "✓ Configuration validated successfully"
    else
        log "✗ Configuration validation failed"
        exit 1
    fi
fi

# Display configuration summary
log "Configuration Summary:"
log "- Hardware PPS: $([ "$ENABLE_PPS" = "yes" ] || [ "$HAS_PPS" = "true" ] && echo "Enabled" || echo "Disabled")"
log "- Hardware GPS: $([ "$ENABLE_GPS" = "yes" ] || [ "$HAS_GPS" = "true" ] && echo "Enabled" || echo "Disabled")"
log "- Network Servers: ${#SERVERS[@]}"
log "- Local Stratum: $LOCAL_STRATUM"
