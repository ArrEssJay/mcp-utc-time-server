# Raspberry Pi Deployment Guide

This guide covers deploying the MCP UTC Time Server on a Raspberry Pi with hardware time sources.

## Hardware Requirements

### Minimum Configuration
- Raspberry Pi 3B+ or newer
- 8GB+ microSD card
- Internet connection (WiFi or Ethernet)

### Recommended Configuration
- Raspberry Pi 4B (2GB+ RAM)
- 16GB+ microSD card (Class 10 or better)
- GPS module with PPS output
- Real-time clock (RTC) module
- External antenna for GPS

## Supported Hardware Time Sources

### 1. GPS with PPS (Pulse Per Second)
- Adafruit Ultimate GPS HAT
- u-blox GPS modules
- Any GPS with NMEA + PPS output

### 2. RTC Modules
- DS3231 (I2C)
- PCF8523 (I2C)
- DS1307 (I2C)

### 3. Rubidium Frequency Standards
- SRS FS725
- FE-5680A
- Symmetricom X72

## OS Setup

### 1. Install Raspberry Pi OS Lite

Download from: https://www.raspberrypi.com/software/operating-systems/

Flash to SD card using:
```bash
# On macOS/Linux
sudo dd if=raspios-lite.img of=/dev/sdX bs=4M status=progress
sync
```

### 2. Enable SSH

```bash
# Mount boot partition and create empty ssh file
touch /Volumes/boot/ssh
```

### 3. Configure WiFi (Optional)

Create `wpa_supplicant.conf` on boot partition:
```
country=US
ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
update_config=1

network={
    ssid="YourNetworkSSID"
    psk="YourPassword"
    key_mgmt=WPA-PSK
}
```

## Kernel Configuration for PPS

### 1. Enable PPS Kernel Module

Edit `/boot/config.txt`:
```bash
# Enable PPS on GPIO4
dtoverlay=pps-gpio,gpiopin=4
```

Or for other GPIO pins:
```bash
# Use GPIO18 instead
dtoverlay=pps-gpio,gpiopin=18
```

### 2. Load PPS Module at Boot

Add to `/etc/modules`:
```
pps-gpio
pps-ldisc
```

### 3. Verify PPS Device

After reboot:
```bash
ls -l /dev/pps*
# Should show: /dev/pps0

# Test PPS signal
sudo ppstest /dev/pps0
# Should show timestamps if GPS has fix
```

## GPS Configuration

### 1. Enable Serial Port

Edit `/boot/config.txt`:
```bash
# Disable Bluetooth to free up serial port
dtoverlay=disable-bt
enable_uart=1
```

### 2. Disable Serial Console

```bash
sudo raspi-config
# Interface Options -> Serial Port
# "Would you like a login shell accessible over serial?" -> No
# "Would you like the serial port hardware to be enabled?" -> Yes
```

### 3. Configure gpsd

Install gpsd:
```bash
sudo apt-get update
sudo apt-get install -y gpsd gpsd-clients
```

Edit `/etc/default/gpsd`:
```bash
DEVICES="/dev/ttyAMA0"
GPSD_OPTIONS="-n"
```

Start gpsd:
```bash
sudo systemctl enable gpsd
sudo systemctl start gpsd
```

Verify:
```bash
cgps -s
# Should show GPS data when satellite fix acquired
```

## RTC Configuration

### 1. Enable I2C

```bash
sudo raspi-config
# Interface Options -> I2C -> Enable
```

### 2. Configure DS3231 RTC

Edit `/boot/config.txt`:
```bash
dtoverlay=i2c-rtc,ds3231
```

### 3. Disable Fake Hardware Clock

```bash
sudo apt-get -y remove fake-hwclock
sudo update-rc.d -f fake-hwclock remove
sudo systemctl disable fake-hwclock
```

### 4. Verify RTC

```bash
sudo i2cdetect -y 1
# Should show device at 0x68

# Read RTC
sudo hwclock -r

# Sync system time to RTC
sudo hwclock -w
```

## Docker Installation

### 1. Install Docker

```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER
```

### 2. Install Docker Compose

```bash
sudo apt-get install -y docker-compose
```

## Deploy MCP Time Server

### 1. Clone Repository

```bash
git clone https://github.com/yourusername/mcp-utc-time-server.git
cd mcp-utc-time-server
```

### 2. Create Environment File

Create `.env`:
```bash
# API Keys
API_KEY_1=your-secure-key-here
API_KEY_2=another-secure-key

# NTP Servers (fallback)
NTP_SERVERS=time.cloudflare.com,time.google.com

# Hardware Time Sources
ENABLE_PPS=yes
PPS_DEVICE=/dev/pps0
PPS_GPIO=4

ENABLE_GPS=yes
GPS_DEVICE=/dev/ttyAMA0
GPS_BAUD=9600

ENABLE_RTC=yes
RTC_DEVICE=/dev/rtc0

# Local Stratum (1 for GPS+PPS)
LOCAL_STRATUM=1

# Log Level
RUST_LOG=info
```

### 3. Deploy with Docker Compose

```bash
docker-compose -f docker-compose.rpi.yml up -d
```

### 4. Verify Deployment

```bash
# Check container status
docker ps

# Check logs
docker logs mcp-utc-time

# Verify NTP sync
docker exec mcp-utc-time ntpq -p

# Test MCP server
curl http://localhost:3000/health
```

## Validation

### 1. Check NTP Synchronization

```bash
docker exec mcp-utc-time ntpq -p
```

Expected output:
```
     remote           refid      st t when poll reach   delay   offset  jitter
==============================================================================
*GPS_NMEA(0)     .GPS.            0 l    7   16  377    0.000   -0.001   0.002
oPPS(0)          .PPS.            0 l    6   16  377    0.000    0.000   0.001
+time1.google    .GOOG.           1 u   32   64  377   12.345    0.123   0.456
```

The `*` indicates the primary time source (GPS), and `o` indicates PPS is being used for disciplining.

### 2. Check System Time Accuracy

```bash
# Check offset from GPS
docker exec mcp-utc-time ntpq -c "rv 0 offset"

# Should be < 1ms with PPS
```

### 3. Test MCP Tools

Using MCP client (e.g., Claude Desktop):

```json
{
  "mcpServers": {
    "utc-time": {
      "command": "docker",
      "args": ["exec", "-i", "mcp-utc-time", "/usr/local/bin/mcp-utc-time-server"]
    }
  }
}
```

## Monitoring

### 1. NTP Statistics

```bash
# View peer statistics
docker exec mcp-utc-time ntpq -p -n

# View system variables
docker exec mcp-utc-time ntpq -c rv

# Check PPS performance
docker exec mcp-utc-time ntpq -c "rv 0 offset,jitter,stratum"
```

### 2. GPS Status

```bash
# Check GPS fix
docker exec mcp-utc-time gpsmon

# Check satellite count
docker exec mcp-utc-time gpspipe -w | grep TPV
```

### 3. System Logs

```bash
# Follow logs
docker logs -f mcp-utc-time

# Check for errors
docker logs mcp-utc-time | grep -i error
```

## Performance Tuning

### 1. CPU Governor

```bash
# Set performance governor for consistent timing
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

### 2. Disable Power Management

Edit `/boot/config.txt`:
```bash
# Disable WiFi power management
dtoverlay=disable-wifi-pm
```

### 3. Increase Process Priority

Edit `docker-compose.rpi.yml`:
```yaml
services:
  mcp-utc-time:
    privileged: true
    # Add nice priority
    init: true
    pid_mode: host
```

## Troubleshooting

### No PPS Signal

```bash
# Check GPIO connection
gpio -g mode 4 in
gpio -g read 4

# Check kernel messages
dmesg | grep pps

# Verify PPS device
sudo ppstest /dev/pps0
```

### GPS Not Working

```bash
# Check serial port
ls -l /dev/ttyAMA0

# Test raw GPS output
sudo cat /dev/ttyAMA0

# Check gpsd status
sudo systemctl status gpsd

# Restart gpsd
sudo systemctl restart gpsd
```

### NTP Not Syncing

```bash
# Check NTP daemon status
docker exec mcp-utc-time ntpq -p

# Check system clock
docker exec mcp-utc-time date

# Restart container
docker-compose -f docker-compose.rpi.yml restart
```

## Auto-start on Boot

### 1. Enable Docker Service

```bash
sudo systemctl enable docker
```

### 2. Configure Restart Policy

Already configured in `docker-compose.rpi.yml`:
```yaml
restart: unless-stopped
```

### 3. Verify Auto-start

```bash
sudo reboot
# Wait for boot
docker ps
# Should show mcp-utc-time running
```

## Security Hardening

### 1. Firewall Configuration

```bash
# Install UFW
sudo apt-get install -y ufw

# Allow SSH
sudo ufw allow 22/tcp

# Allow MCP (if remote access needed)
sudo ufw allow 3000/tcp

# Enable firewall
sudo ufw enable
```

### 2. Rotate API Keys

Update `.env` file and restart:
```bash
docker-compose -f docker-compose.rpi.yml down
# Update .env
docker-compose -f docker-compose.rpi.yml up -d
```

### 3. Update System

```bash
sudo apt-get update
sudo apt-get upgrade -y
sudo apt-get autoremove -y
```

## Backup and Recovery

### 1. Backup Configuration

```bash
# Backup .env and compose file
tar czf mcp-backup-$(date +%Y%m%d).tar.gz \
  .env docker-compose.rpi.yml
```

### 2. SD Card Image

```bash
# On another machine, create image
sudo dd if=/dev/sdX of=rpi-mcp-backup.img bs=4M status=progress
gzip rpi-mcp-backup.img
```

## Fleet Deployment

For managing multiple Raspberry Pi devices, see `docs/FLEET_MANAGEMENT.md`.
