# megaton-lib
User facing functions of megaton, for use when writing mods

  "command": "/opt/devkitpro/devkitA64/bin/aarch64-none-elf-g++ 
  -DMEGART_NX_MODULE_NAME=\\\"test\\\" 
  -DMEGART_NX_MODULE_NAME_LEN=4 
  -DMEGART_TITLE_ID=0 
  -DMEGART_TITLE_ID_HEX=\\\"0x0\\\" 
  -DMEGATON_LIB 
  -DNNSDK 
  -DSWITCH 
  -D__SWITCH__ 
  -I/home/piston/dev/megaton/packages/lib/include 
  -I/home/piston/dev/megaton/packages/lib/../nnheaders/include 
  -isystem /opt/devkitpro/libnx/include 
  -std=c++20 
  -march=armv8-a+crc+crypto 
  -mtune=cortex-a57 
  -mtp=soft 
  -fPIC 
  -fvisibility=hidden -g -Wall -Werror -ffunction-sections -fdata-sections -O2 -fno-rtti 
  -fno-exceptions -fno-asynchronous-unwind-tables -fno-unwind-tables -fmodules-ts -fmodule-mapper=CMakeFiles/megaton.dir/src/nximpl/random.cpp.obj.modmap -MD 
  -fdeps-format=p1689r5 -x c++ -o CMakeFiles/megaton.dir/src/nximpl/random.cpp.obj -c /home/piston/dev/megaton/packages/lib/src/nximpl/random.cpp",
