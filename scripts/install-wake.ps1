# scripts/install-wake.ps1
# Skrypt konfigurujący zadanie w Harmonogramie Zadań (Task Scheduler) z flagą WakeToRun
# Uruchamiaj w PowerShell (jako Administrator)

$ErrorActionPreference = "Stop"

$ConfigDir = "$env:APPDATA\com.app.waste-reminder"
if (-not (Test-Path $ConfigDir)) {
    Write-Error "Katalog konfiguracji nie istnieje: $ConfigDir. Uruchom aplikację najpierw."
    exit 1
}

$ConfigPath = Join-Path $ConfigDir "config.json"
$AlertTime = "06:30"
if (Test-Path $ConfigPath) {
    try {
        $cfg = Get-Content $ConfigPath -Raw | ConvertFrom-Json
        if ($cfg.alert_time) {
            $AlertTime = $cfg.alert_time.Substring(0, 5)
        }
    } catch {
        Write-Warning "Nie udało się odczytać alert_time z config.json, używam domyślnego $AlertTime"
    }
}

# 5 minut przed alert_time
$parsedTime = [DateTime]::ParseExact($AlertTime, "HH:mm", [System.Globalization.CultureInfo]::InvariantCulture)
$wakeTime = $parsedTime.AddMinutes(-5).ToString("HH:mm:ss")

Write-Host "Godzina wybudzania: $wakeTime (5 min przed $AlertTime)"

# Pobierz nadchodzące daty
$now = [DateTime]::Today
$scheduleFiles = Get-ChildItem -Path $ConfigDir -Filter "trash_schedule_*.json"
$dates = [System.Collections.Generic.HashSet[DateTime]]::new()

foreach ($file in $scheduleFiles) {
    try {
        $data = Get-Content $file.FullName -Raw | ConvertFrom-Json
        foreach ($prop in $data.fractions.PSObject.Properties) {
            foreach ($dStr in $prop.Value.dates) {
                $d = [DateTime]::ParseExact($dStr, "yyyy-MM-dd", [System.Globalization.CultureInfo]::InvariantCulture)
                if ($d -ge $now) {
                    [void]$dates.Add($d)
                }
            }
        }
    } catch {
        Write-Warning "Błąd przetwarzania $($file.Name): $_"
    }
}

$sortedDates = $dates | Sort-Object | Select-Object -First 20

if ($sortedDates.Count -eq 0) {
    Write-Host "Brak nadchodzących dat wywozu odpadów."
    exit 0
}

Write-Host "Znaleziono $($sortedDates.Count) nadchodzących dat do zaplanowania wybudzenia."

$TaskName = "WasteReminderWake"

# Usunięcie poprzedniego zadania jeśli istnieje
try {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
} catch {}

$triggers = @()
foreach ($d in $sortedDates) {
    $dt = [DateTime]::ParseExact("$($d.ToString('yyyy-MM-dd')) $wakeTime", "yyyy-MM-dd HH:mm:ss", [System.Globalization.CultureInfo]::InvariantCulture)
    $triggers += New-ScheduledTaskTrigger -Once -At $dt
}

$action = New-ScheduledTaskAction -Execute "cmd.exe" -Argument "/c timeout /t 10"
$settings = New-ScheduledTaskSettingsSet -WakeToRun -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $triggers -Settings $settings -Description "Wybudzanie systemu przed alarmem wywozu odpadów (Waste Reminder)"

Write-Host "Pomyślnie utworzono zadanie harmonogramu '$TaskName' z flagą WakeToRun."
