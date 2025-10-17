#!/bin/bash
# Container Entrypoint for MCP Time Server
# Handles hardware detection, NTP configuration, and service startup

set -e

echo "========================================"
echo "MCP UTC Time Server - Hardware Edition"
echo "========================================"
echo "Starting at: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Check kernel capabilities
check_capabilities() {
    echo "Checking kernel capabilities..."
    
    # Check for SYS_TIME (required for time adjustment)
    if capsh --print 2>/dev/null | grep -q cap_sys_time; then
        echo "✓ SYS_TIME capability available"
    else
        echo "⚠ SYS_TIME capability missing - time adjustment may be limited"
    fi
    
    # Check for device access
    if [ -w /dev/pps0 ] 2>/dev/null; then
        echo "✓ PPS device accessible (/dev/pps0)"
    fi
    
    if [ -w /dev/gpiomem ] 2>/dev/null; then
        echo "✓ GPIO memory accessible"
    fi
    
    echo ""
}

# Load required kernel modules
load_modules() {
    echo "Loading kernel modules..."
    
    # PPS modules
    modprobe pps-core 2>/dev/null || echo "⚠ pps-core module not available"
    modprobe pps-gpio gpio_pin="${PPS_GPIO:-4}" 2>/dev/null || echo "⚠ pps-gpio module not available"
    modprobe pps-ldisc 2>/dev/null || echo "⚠ pps-ldisc module not available"
    
    echo ""
}

# Configure system for time accuracy
configure_system() {
    echo "Configuring system for accurate timekeeping..."
    
    # Set system clock source to highest resolution available
    if [ -f /sys/devices/system/clocksource/clocksource0/current_clocksource ]; then
        CURRENT=$(cat /sys/devices/system/clocksource/clocksource0/current_clocksource)
        echo "Current clocksource: $CURRENT"
        
        # Try to use TSC if available
        if [ -f /sys/devices/system/clocksource/clocksource0/available_clocksource ]; then
            if grep -q tsc /sys/devices/system/clocksource/clocksource0/available_clocksource; then
                echo tsc > /sys/devices/system/clocksource/clocksource0/current_clocksource 2>/dev/null || true
            fi
        fi
    fi
    
    echo ""
}

# Wait for hardware initialization
wait_for_hardware() {
    echo "Waiting for hardware initialization..."
    
    local max_wait=30
    local count=0
    
    while [ $count -lt $max_wait ]; do
        if [ "$ENABLE_PPS" = "yes" ] || [ "$ENABLE_PPS" = "auto" ]; then
            if [ -c /dev/pps0 ]; then
                echo "✓ PPS device ready"
                break
            fi
        else
            break
        fi
        
        sleep 1
        count=$((count + 1))
    done
    
    echo ""
}

# Main execution flow
main() {
    check_capabilities
    
    # Load modules if running in privileged mode
    if [ "$(id -u)" = "0" ]; then
        load_modules
        configure_system
    fi
    
    wait_for_hardware
    
    # Configure NTP
    echo "Configuring NTPsec..."
    /usr/local/bin/configure-ntp.sh
    echo ""
    
    # Display final status
    echo "========================================"
    echo "Configuration Complete"
    echo "========================================"
    echo "NTP Configuration: /etc/ntpsec/ntp.conf"
    echo "Hardware Detection: /usr/local/bin/detect-hardware.sh"
    echo ""
    echo "Starting services via supervisord..."
    echo ""
    
    # Start supervisor (which will start ntpd and mcp-server)
    exec /usr/bin/supervisord -c /etc/supervisor/supervisord.conf
}

main "$@"
