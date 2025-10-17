#!/bin/sh
# Hardware Detection Script for Time Sources
# Detects PPS, GPS, RTC, and other timing hardware

set -e

detect_pps() {
    # Check for PPS device
    if [ -c /dev/pps0 ]; then
        echo "found:pps:/dev/pps0"
        
        # Check GPIO pin assignment
        if [ -f /sys/class/pps/pps0/path ]; then
            PPS_PATH=$(cat /sys/class/pps/pps0/path)
            echo "pps:path:$PPS_PATH"
        fi
        
        # Test PPS signal
        if command -v ppstest >/dev/null 2>&1; then
            if timeout 2 ppstest /dev/pps0 -c 1 >/dev/null 2>&1; then
                echo "pps:signal:ok"
            else
                echo "pps:signal:none"
            fi
        fi
    fi
}

detect_gps() {
    # Check for GPS serial devices
    for device in /dev/ttyAMA0 /dev/ttyUSB0 /dev/ttyS0 /dev/ttyACM0; do
        if [ -c "$device" ]; then
            # Try to read NMEA data
            stty -F "$device" 9600 cs8 -cstopb -parenb 2>/dev/null || continue
            
            if timeout 2 cat "$device" 2>/dev/null | grep -q '^\$GP\|^\$GN'; then
                echo "found:gps:$device"
                
                # Try to get fix status
                FIX=$(timeout 2 cat "$device" 2>/dev/null | grep '^\$GPGGA' | head -1)
                if echo "$FIX" | grep -q ',1,' || echo "$FIX" | grep -q ',2,'; then
                    echo "gps:fix:ok"
                else
                    echo "gps:fix:none"
                fi
            fi
        fi
    done
}

detect_rtc() {
    # Check for hardware RTC
    if [ -c /dev/rtc0 ]; then
        echo "found:rtc:/dev/rtc0"
        
        if command -v hwclock >/dev/null 2>&1; then
            if hwclock --show >/dev/null 2>&1; then
                echo "rtc:status:ok"
                RTC_TIME=$(hwclock --get)
                echo "rtc:time:$RTC_TIME"
            else
                echo "rtc:status:error"
            fi
        fi
    fi
}

detect_rubidium() {
    # Check for Rubidium/OCXO frequency standards
    for device in /dev/ttyUSB* /dev/ttyACM*; do
        [ -c "$device" ] || continue
        
        # Try common Rubidium protocols
        stty -F "$device" 9600 2>/dev/null || continue
        
        # Send status query
        echo "STATUS?" > "$device" 2>/dev/null || continue
        
        if timeout 1 cat "$device" 2>/dev/null | grep -qi 'LOCK\|RUBIDIUM\|OCXO'; then
            echo "found:rubidium:$device"
        fi
    done
}

detect_system_clock() {
    # Check system clock capabilities
    if [ -f /sys/devices/system/clocksource/clocksource0/current_clocksource ]; then
        CLOCKSOURCE=$(cat /sys/devices/system/clocksource/clocksource0/current_clocksource)
        echo "system:clocksource:$CLOCKSOURCE"
    fi
    
    # Check for available clocksources
    if [ -f /sys/devices/system/clocksource/clocksource0/available_clocksource ]; then
        AVAILABLE=$(cat /sys/devices/system/clocksource/clocksource0/available_clocksource)
        echo "system:available_clocksources:$AVAILABLE"
    fi
}

# Main detection routine
main() {
    echo "=== Hardware Time Source Detection ==="
    echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo ""
    
    detect_pps
    detect_gps
    detect_rtc
    detect_rubidium
    detect_system_clock
    
    echo ""
    echo "=== Detection Complete ==="
}

main "$@"
