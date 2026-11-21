#!/usr/bin/env bash
set -e

OS="$(uname -s)"

if [ "$OS" = "Darwin" ]; then
    echo "=== Waste Reminder: Konfiguracja wybudzania dla macOS (pmset) ==="
    
    CONFIG_DIR="${CONFIG_DIR:-$HOME/Library/Application Support/com.app.waste-reminder}"
    if [ ! -d "$CONFIG_DIR" ]; then
        echo "Błąd: Katalog konfiguracji nie istnieje: $CONFIG_DIR"
        echo "Upewnij się, że aplikacja została uruchomiona przynajmniej raz i dane zostały skopiowane."
        exit 1
    fi

    python3 -c "
import os, glob, json, sys, subprocess
from datetime import datetime, timedelta, time

config_dir = os.path.expanduser('$CONFIG_DIR')
cfg_path = os.path.join(config_dir, 'config.json')

alert_hour, alert_minute = 6, 30
if os.path.exists(cfg_path):
    try:
        with open(cfg_path, 'r', encoding='utf-8') as f:
            cfg = json.load(f)
            t_str = cfg.get('alert_time', '06:30:00')
            parts = [int(p) for p in t_str.split(':')[:2]]
            alert_hour, alert_minute = parts[0], parts[1]
    except Exception as e:
        print(f'Uwaga: Nie można odczytać alert_time z config.json: {e}')

now = datetime.now()
target_time = (datetime.combine(now.date(), time(alert_hour, alert_minute)) - timedelta(minutes=5)).time()

dates = set()
for path in glob.glob(os.path.join(config_dir, 'trash_schedule_*.json')):
    try:
        with open(path, 'r', encoding='utf-8') as f:
            data = json.load(f)
            for frac in data.get('fractions', {}).values():
                for d_str in frac.get('dates', []):
                    try:
                        d = datetime.strptime(d_str, '%Y-%m-%d').date()
                        if d >= now.date():
                            dates.add(d)
                    except ValueError:
                        pass
    except Exception as e:
        print(f'Błąd czytania {path}: {e}')

sorted_dates = sorted(list(dates))[:20]

if not sorted_dates:
    print('Brak nadchodzących dat wywozu odpadów w harmonogramach.')
    sys.exit(0)

print(f'Godzina wybudzania: {target_time.strftime(\"%H:%M:%S\")} (5 min przed {alert_hour:02d}:{alert_minute:02d})')
print(f'Znaleziono {len(sorted_dates)} nadchodzących dat (maks 20).')

# Anuluj poprzednie wpisy pmset dla tego typu
try:
    out = subprocess.check_output(['pmset', '-g', 'sched'], text=True)
    for line in out.splitlines():
        if 'wakeorpoweron' in line:
            parts = line.strip().split()
            for i, p in enumerate(parts):
                if '/' in p and i+1 < len(parts) and ':' in parts[i+1]:
                    dt_str = f'{p} {parts[i+1]}'
                    subprocess.run(['sudo', 'pmset', 'schedule', 'cancel', 'wakeorpoweron', dt_str], capture_output=True)
except Exception:
    pass

for d in sorted_dates:
    dt = datetime.combine(d, target_time)
    formatted = dt.strftime('%m/%d/%y %H:%M:%S')
    print(f'Dodawanie wybudzania: {formatted}')
    res = subprocess.run(['sudo', 'pmset', 'schedule', 'wakeorpoweron', formatted])
    if res.returncode != 0:
        print(f'Ostrzeżenie: Błąd podczas planowania pmset dla {formatted}')

print('Pomyślnie zaktualizowano harmonogram wybudzania systemu macOS.')
"

elif [ "$OS" = "Linux" ]; then
    echo "=== Waste Reminder: Konfiguracja wybudzania dla Linux (systemd-timer / rtcwake) ==="
    CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/com.app.waste-reminder}"
    if [ ! -d "$CONFIG_DIR" ]; then
        echo "Błąd: Katalog konfiguracji nie istnieje: $CONFIG_DIR"
        exit 1
    fi

    python3 -c "
import os, glob, json, sys, subprocess
from datetime import datetime, timedelta, time

config_dir = os.path.expanduser('$CONFIG_DIR')
cfg_path = os.path.join(config_dir, 'config.json')

alert_hour, alert_minute = 6, 30
if os.path.exists(cfg_path):
    try:
        with open(cfg_path, 'r', encoding='utf-8') as f:
            cfg = json.load(f)
            t_str = cfg.get('alert_time', '06:30:00')
            parts = [int(p) for p in t_str.split(':')[:2]]
            alert_hour, alert_minute = parts[0], parts[1]
    except Exception as e:
        pass

now = datetime.now()
target_time = (datetime.combine(now.date(), time(alert_hour, alert_minute)) - timedelta(minutes=5)).time()

dates = set()
for path in glob.glob(os.path.join(config_dir, 'trash_schedule_*.json')):
    try:
        with open(path, 'r', encoding='utf-8') as f:
            data = json.load(f)
            for frac in data.get('fractions', {}).values():
                for d_str in frac.get('dates', []):
                    try:
                        d = datetime.strptime(d_str, '%Y-%m-%d').date()
                        if d >= now.date():
                            dates.add(d)
                    except ValueError:
                        pass
    except Exception:
        pass

sorted_dates = sorted(list(dates))[:20]
if not sorted_dates:
    print('Brak nadchodzących dat wywozu.')
    sys.exit(0)

service_content = '''[Unit]
Description=Waste Reminder Wake Service

[Service]
Type=oneshot
ExecStart=/bin/true
'''

timer_lines = ['[Unit]', 'Description=Waste Reminder Wake Timer', '', '[Timer]', 'WakeSystem=true']
for d in sorted_dates:
    dt_str = f'{d.strftime(\"%Y-%m-%d\")} {target_time.strftime(\"%H:%M:%S\")}'
    timer_lines.append(f'OnCalendar={dt_str}')

timer_lines.append('Persistent=false')
timer_content = '\n'.join(timer_lines) + '\n'

systemd_dir = os.path.expanduser('~/.config/systemd/user')
os.makedirs(systemd_dir, exist_ok=True)

with open(os.path.join(systemd_dir, 'waste-reminder-wake.service'), 'w') as f:
    f.write(service_content)

with open(os.path.join(systemd_dir, 'waste-reminder-wake.timer'), 'w') as f:
    f.write(timer_content)

subprocess.run(['systemctl', '--user', 'daemon-reload'])
subprocess.run(['systemctl', '--user', 'enable', '--now', 'waste-reminder-wake.timer'])
print('Pomyślnie zainstalowano timer systemd z WakeSystem=true dla Linux.')
"
else
    echo "Nieobsługiwany system operacyjny dla tego skryptu: $OS"
    exit 1
fi
