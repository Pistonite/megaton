#!/usr/bin/ruby

# SPDX-License-Identifier: MIT
# Copyright (c) 2025 Megaton contributors

require 'getoptlong'

prefix = 'packages/'
# location of c/c++ source files
cxx_dirs = [
  'sys',
  'lib',
  'nx',
  'abi'
]
# location of rust source files
rust_dirs = [
  'lib',
  'nx',
  'abi'
]

# default cmdline option directories
output = '../megaton-old'
mod = '../megaton-example'
yuzu = '~/.local/share/eden'

user_common_flags = "-I#{mod}/packages/botw-symbols/include/ -DBOTWTOOLKIT_TCP_SEND"
user_as_flags = ''
user_c_flags = ''
user_cxx_flags = ''
user_cargo_flags = ''
task_exe = 'go-task'

# compilers
cc = '/opt/devkitpro/devkitA64/bin/aarch64-none-elf-gcc'
cxx = '/opt/devkitpro/devkitA64/bin/aarch64-none-elf-g++'
ar = '/opt/devkitpro/devkitA64/bin/aarch64-none-elf-ar'

# parse commandline options
opts = GetoptLong.new(
  ['--output', '-o', GetoptLong::OPTIONAL_ARGUMENT],
  ['--mod', '-m', GetoptLong::OPTIONAL_ARGUMENT],
  ['--yuzu', '-y', GetoptLong::OPTIONAL_ARGUMENT],
  ['--help', '-h', GetoptLong::NO_ARGUMENT],
  ['--c-flags', GetoptLong::REQUIRED_ARGUMENT],
  ['--as-flags', GetoptLong::REQUIRED_ARGUMENT],
  ['--cxx-flags', GetoptLong::REQUIRED_ARGUMENT],
  ['--com-flags', GetoptLong::REQUIRED_ARGUMENT],
  ['--rs-flags', GetoptLong::REQUIRED_ARGUMENT],
  ['--task', '-t', GetoptLong::REQUIRED_ARGUMENT],
  ['--clean', '-x', GetoptLong::NO_ARGUMENT]
)

do_output = false
do_mod = false
do_yuzu = false

# formatting colors
red_bold = "\e[31m\e[1m"
green_bold = "\e[32m\e[1m"
reset = "\e[0m"

# some of the cmdline options (see end of file)
opts.each do |opt, arg|
  case opt
  when '--help'
    puts <<-EOF
./build.rb
Build libmegaton.a to build/a/libmegaton.a. The below options are in addition 
to building to library (ex: build library, then copy it)
  -h, --help
    Print this help
  -o, --output [DIR]
    Copy libmegaton.a to megaton-old, which is located at DIR. Defaults to ../megaton-old/
  -m, --mod [DIR]
    Compile example mod, which is at DIR. Defaults to ../megaton-example/
  -y, --yuzu [DIR]
    Copy example mod to yuzu (or eden) folder, which is at DIR. Defaults to ~/.local/share/eden/
  --c-flags FLAGS
    Add flags to c compiler
  --cxx-flags FLAGS
    Add flags to cpp compiler
  --as-flags FLAGS
    Add flags to assembler
  --com-flags FLAGS
    Add flags to c, cpp, assembly
  --rs-flags FLAGS
    Add flags to cargo (build)
  -t, --task EXECUTABLE
    Change task executable to EXECUTABLE, purposes of building the example mod
  -x, --clean
    Remove builds and run cargo clean
  EOF
  exit(0)
  when '--clean'
    `rm -rf build`
    `cargo clean`
    exit(0)
  when '--com-flags'
    user_common_flags = "#{user_common_flags} #{arg}"
  when '--c-flags'
    user_c_flags = "#{user_c_flags} #{arg}"
  when '--as-flags'
    user_as_flags = "#{user_as_flags} #{arg}"
  when '--cxx-flags'
    user_cxx_flags = "#{user_cxx_flags} #{arg}"
  when '--rs-flags'
    user_cargo_flags = "#{user_cargo_flags} #{arg}"
  when '--task'
    task_exe = arg
  when '--output'
    do_output = true
    unless arg.empty?
      output = arg
    end
  when '--mod'
    do_mod = true
    unless arg.empty?
      mod = arg
    end
  when '--yuzu'
    do_yuzu = true
    unless arg.empty?
      yuzu = arg
    end
  end
end

# tweaking paths to be more accurate
cxx_dirs = cxx_dirs.map { |x| unless x == 'sys' 
                          "#{prefix}#{x}/cxx" else "#{prefix}#{x}" end }
rust_dirs = rust_dirs.map { |x| "#{prefix}#{x}/rs" }
pwd = `pwd`.strip

# create all c/c++ source file paths
cxx_source_dirs = cxx_dirs.map { |x| "#{x}/src" }
cxx_bridge_source = 'build/cxxbridge/src'
cxx_source_dirs.push(cxx_bridge_source)

# create all c/c++ include paths
cxx_include_dirs = cxx_dirs.map { |x| "#{x}/include" }
cxx_bridge_include = 'build/cxxbridge/include'
cxx_include_dirs.push(cxx_bridge_include)
devkitpro_include = '/opt/devkitpro/libnx/include'

# convert paths to include flags
include_path = cxx_include_dirs.map { |x| "#{pwd}/#{x}"}
include_path.push(devkitpro_include)
include_flags = include_path.map { |p| "-I#{p}"}

# create any directories that might not already exist
include_path.each do |p|
  `mkdir -p #{p}`
end
cxx_source_dirs.each do |p|
  `mkdir -p #{p}`
end
rust_dirs.each do |p|
  `mkdir -p #{p}` 
end

# compiler flags
common_flags="-march=armv8-a+crc+crypto -mtune=cortex-a57 -mtp=soft -fPIC -fvisibility=hidden -g #{include_flags.join(' ')} #{user_common_flags}"
c_flags="-Wall -Werror -O3 -fdiagnostics-color=always -DMEGATON_LIB -xc #{user_c_flags}"
cxx_flags="-Wall -Werror -O3 -fdiagnostics-color=always -DMEGATON_LIB -std=c++20 -fno-rtti -fno-exceptions -fno-asynchronous-unwind-tables -fno-unwind-tables #{user_cxx_flags}"
as_flags="-x assembler-with-cpp -Wall -Werror -O3 -fdiagnostics-color=always -DMEGATON_LIB -std=c++20 -fno-rtti -fno-exceptions -fno-asynchronous-unwind-tables -fno-unwind-tables #{user_as_flags}"

# get all rust files with cxxbridge
rust_src_dirs = rust_dirs.map{ |p| "#{p}/src"}
cxx_query = '#\\[cxx::bridge\\]'
cxx_rs = `grep -rl '#{cxx_query}' #{rust_src_dirs.join(' ')}`.split("\n")

# run cxxbridge on each rust cxx file
puts "#{green_bold}creating cxx headers/.cc files#{reset}"
cxx_rs.each do |f|
  output_path = f.split('/')
  output_path.delete_at(0)
  output_path.slice!(1..2)
  output_name = output_path.pop
  `mkdir -p build/cxxbridge/include/#{output_path.join('/')}`
  `mkdir -p build/cxxbridge/src/#{output_path.join('/')}`
  `cxxbridge #{f} --header > build/cxxbridge/include/#{output_path.join('/')}/#{output_name}.h`
  unless Process.last_status.success?
    puts "#{red_bold}Error running 'cxxbridge --header' on #{f}. Exiting...#{reset}"
    `rm -rf build`
    exit(-1)
  end
  `cxxbridge #{f} > build/cxxbridge/src/#{output_path.join('/')}/#{output_name}.cc`
  unless Process.last_status.success?
    puts "#{red_bold}Error running 'cxxbridge' on #{f}. Exiting...#{reset}"
    `rm -rf build`
    exit(-1)
  end
end
`mkdir -p build/cxxbridge/include/rust/`
`cxxbridge --header > build/cxxbridge/include/rust/cxx.h`
unless Process.last_status.success?
  puts "#{red_bold}Error running 'cxxbridge --header'. Exiting...#{reset}"
  `rm -rf build`
  exit(-1)
end
puts "#{green_bold}Successfully created cxxbridge headers and .cc files#{reset}"
puts ''

# find all as/c/c++ source files
as_files = `find #{cxx_source_dirs.join(' ')} -iname '*.s'`.split("\n")
c_files = `find #{cxx_source_dirs.join(' ')} -iname '*.c'`.split("\n")
cpp_files = `find #{cxx_source_dirs.join(' ')} \\( -iname '*.cpp' -o -iname '*.cc' \\)`.split("\n")

# (re)create object and staticlib directories
`rm -r build/a 2>/dev/null`
`rm -r build/o 2>/dev/null`
`mkdir -p build/o`
`mkdir -p build/a`

# build as/c/c++ files into objects
puts "#{green_bold}compiling assembly files#{reset}"
as_files.each do |f|
  output_path = f.split('/')
  output_path.delete('packages')
  output_path.delete('src')
  output_path.delete('build')
  output_name = output_path.pop
  output_path = output_path.join('/')
  `mkdir -p build/o/#{output_path}`
  `#{cxx} #{common_flags} #{as_flags} -c #{f} -o build/o/#{output_path}/#{output_name}.o`
  unless Process.last_status.success?
    puts "#{red_bold}Error building #{f} with g++. Exiting...#{reset}"
    `rm -rf build`
    exit(-1)
  end
end
puts "#{green_bold}Successfully built assembly files into objects#{reset}"
puts ''

puts "#{green_bold}compiling c files#{reset}"
c_files.each do |f|
  output_path = f.split('/')
  output_path.delete('packages')
  output_path.delete('src')
  output_path.delete('build')
  output_name = output_path.pop
  output_path = output_path.join('/')
  `mkdir -p build/o/#{output_path}`
  `#{cc} #{common_flags} #{c_flags} -c #{f} -o build/o/#{output_path}/#{output_name}.o`
  unless Process.last_status.success?
    puts "#{red_bold}Error building #{f} with gcc. Exiting...#{reset}"
    `rm -rf build`
    exit(-1)
  end
end
puts "#{green_bold}Successfully built c files into objects#{reset}"
puts ''

puts "#{green_bold}compiling c++ files#{reset}"
cpp_files.each do |f|
  output_path = f.split('/')
  output_path.delete('packages')
  output_path.delete('src')
  output_path.delete('build')
  output_name = output_path.pop
  output_path = output_path.join('/')
  `mkdir -p build/o/#{output_path}`
  `#{cxx} #{common_flags} #{cxx_flags} -c #{f} -o build/o/#{output_path}/#{output_name}.o`
  unless Process.last_status.success?
    puts "#{red_bold}Error building #{f} with g++. Exiting...#{reset}"
    `rm -rf build`
    exit(-1)
  end
end
puts "#{green_bold}Successfully built c++ files into objects#{reset}"
puts ''

# build rust projects
puts "#{green_bold}compiling rust projects#{reset}"
rust_dirs.each do |f|
  `cd #{f}; cargo +megaton build --release --target=aarch64-unknown-hermit`
  unless Process.last_status.success?
    puts "#{red_bold}Error building #{f} with cargo. Exiting...#{reset}"
    `rm -rf build`
    `cargo clean`
    exit(-1)
  end
end
puts "#{green_bold}Successfully built rust packages#{reset}"
puts ''

# compile object files into libmegaton_c.a
puts "#{green_bold}creating libmegaton_c.a#{reset}"
obj_files = `find build/o -iname '*.o'`.split("\n")
`#{ar} rcs build/a/libmegaton_c.a #{obj_files.join(' ')}`
puts "#{green_bold}libmegaton_c.a created successfully#{reset}"
puts ''

# copy libmegaton_rs.a and create libmegaton.a
puts "#{green_bold}creating libmegaton.a#{reset}"
`cp target/aarch64-unknown-hermit/release/libmegaton.a build/a/libmegaton_rs.a`
`#{ar} cqT build/a/libmegaton.a build/a/libmegaton_c.a build/a/libmegaton_rs.a`
unless Process.last_status.success?
  puts "#{red_bold}Error creating libmegaton.a thin archive. Exiting...#{reset}"
  `rm -rf build`
  `cargo clean`
  exit(-1)
end
`echo -e 'create build/a/libmegaton.a\\naddlib build/a/libmegaton.a\\nsave\\nend' | #{ar} -M`
unless Process.last_status.success?
  puts "#{red_bold}Error building #{f} with cargo. Exiting...#{reset}"
  `rm -rf build`
  `cargo clean`
  exit(-1)
end
puts "#{green_bold}successfully created libmegaton.a#{reset}"
puts ''

# act on the cmdline options
if do_output
  puts "#{green_bold}copying library to old megaton...#{reset}"
  `cp build/a/libmegaton.a #{output}/lib/build/bin/libmegaton.a`
end
if do_mod
  puts "#{green_bold}building example mod#{reset}"
  `cd #{mod}/packages/example-mod/; #{task_exe} build`
  unless Process.last_status.success?
    puts "#{red_bold}Error building example mod. Exiting...#{reset}"
    `rm -rf build`
    `cargo clean`
    exit(-1)
  end
  puts "#{green_bold}Example mod build success#{reset}"
end
if do_yuzu
  puts "#{green_bold}copying mod to eden/yuzu#{reset}"
  `cp #{mod}/packages/example-mod/target/megaton/none/example.nso #{yuzu}/sdmc/atmosphere/contents/01007EF00011E000/exefs/subsdk9`
end
