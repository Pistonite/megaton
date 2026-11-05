lcd target
lcd crash_reports
mget -E /atmosphere/crash_reports/*
lcd dumps
mget -E /atmosphere/crash_reports/dumps/*
rm -rf /atmosphere/crash_reports/*
lcd ../../logs
mget -E /megaton_logs.txt
