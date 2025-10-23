# NTPsec Shared Memory Setup

This server uses the NTPsec shared memory driver to communicate with NTP for hardware clock support (including GPIO PPS signals on Raspberry Pi).

## Configuration

### NTPsec Configuration

Add to your `/etc/ntpsec/ntp.conf`:

```conf
# Shared memory driver for MCP UTC Time Server
# This allows the server to read NTP timing data
server 127.127.28.0 mode 1 minpoll 4 maxpoll 4 prefer
fudge 127.127.28.0 refid SHM0

# For PPS (Pulse Per Second) support on Raspberry Pi GPIO:
# Uncomment if you have a GPS with PPS connected
# server 127.127.28.1 minpoll 4 maxpoll 4 prefer
# fudge 127.127.28.1 refid PPS flag3 1

# Standard NTP pool servers
pool pool.ntp.org iburst
```

### Shared Memory Units

The server connects to SHM unit 0 by default (corresponds to `127.127.28.0` in NTPsec config).

- **SHM(0)**: System time from NTP
- **SHM(1)**: PPS (Pulse Per Second) from GPS GPIO
- **SHM(2)**: Additional time source
- **SHM(3)**: Additional time source

### Permissions

The shared memory segments are created with permissions `0666`, allowing the server to read from them without root privileges.

To verify shared memory segments are created:

```bash
ipcs -m | grep 4e54
```

You should see entries like:
```
0x4e545030  123456  ntpsec    666   96
```

## Hardware Clock (PPS) Support

### Raspberry Pi GPIO PPS

1. **Enable PPS kernel module:**

Edit `/boot/config.txt`:
```ini
# Enable PPS on GPIO 18
dtoverlay=pps-gpio,gpiopin=18
```

2. **Load kernel module:**

```bash
sudo modprobe pps-gpio
```

Add to `/etc/modules`:
```
pps-gpio
```

3. **Verify PPS device:**

```bash
ls -l /dev/pps*
# Should show: /dev/pps0

# Test PPS signal
sudo ppstest /dev/pps0
```

4. **Configure NTPsec for PPS:**

Add to `/etc/ntpsec/ntp.conf`:
```conf
# PPS reference clock
server 127.127.28.1 minpoll 4 maxpoll 4 prefer
fudge 127.127.28.1 refid PPS flag3 1
```

### Checking PPS Status

Once configured, the MCP server will report PPS status in the `get_ntp_status` tool:

```json
{
  "pps_enabled": true,
  "hardware_clock": "PPS active",
  "shm_valid": true
}
```

## Troubleshooting

### SHM not connected

If `shm_interface: "disconnected"`:

1. Check NTPsec is running:
   ```bash
   sudo systemctl status ntpsec
   ```

2. Verify SHM segments exist:
   ```bash
   ipcs -m | grep 4e54
   ```

3. Check NTPsec configuration:
   ```bash
   sudo ntpq -p
   ```

### PPS not detected

If `pps_enabled: false`:

1. Verify PPS device exists:
   ```bash
   ls /dev/pps*
   ```

2. Check PPS is receiving pulses:
   ```bash
   sudo ppstest /dev/pps0
   ```

3. Verify NTPsec sees PPS:
   ```bash
   sudo ntpq -c peers
   # Look for PPS refid
   ```

## Performance

The shared memory interface provides:
- **Latency**: < 1µs (compared to ~10ms for command-line tools)
- **Precision**: Nanosecond resolution when available
- **Overhead**: Minimal CPU usage (simple memory read)
- **Hardware support**: Full GPIO PPS support on Raspberry Pi

## Security

The shared memory segments are read-only from the server's perspective. The server cannot modify NTP's behavior or timing data.

For production deployments:
- Run the server as a non-privileged user
- NTPsec runs as `ntpsec:ntpsec` user
- Shared memory permissions allow read access without escalation
