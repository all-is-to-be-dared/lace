#!/bin/bash

version="0.2.0"

nice_path() {
  perl -le "use File::Spec;print File::Spec->abs2rel(@ARGV)" "$1" "$(pwd)"
}

self_raw="$0"
self=$(nice_path "$self_raw")

usage() {
  echo
  echo "Usage:"
  echo -e "\t$self\t[-hv]\t<FILE>"
}

error_usage() {
  usage
  echo "For more information, invoke this script with the '-h' option."
  exit 1
}

help() {
  echo "Upload a RISC-V ELF binary using XFEL."
  usage
  echo
  echo "Options:"
  echo -e "\t-h, --help\tDisplay this help information"
  echo -e "\t-v, --version\tDisplay version of this script"
}

version() {
  echo "upload-d1.sh version 0.2.0"
}

if [[ $# -ne 1 ]] ; then
  echo "$self: Expected one parameter"
  error_usage
fi

if [[ "x$1" == "x-h" || "x$1" == "x--help" ]] ; then
  help
  exit 0
fi
if [[ "x$1" == "x-v" || "x$1" == "x--version" ]] ; then
  version
  exit 0
fi

if ! [[ -e "$1" ]] ; then
  echo "$self: path '$1' must exist"
  exit 1
fi
if ! [[ -f "$1" ]] ; then
  echo "$self: path '$1' must be a file"
  exit 1
fi

elf_path="$1"
target_dir="$(nice_path "$(dirname "${elf_path}")")"
base_name="$(basename "${elf_path}")"
bin_path="${target_dir}/${base_name/.elf/}.bin"

llvm-objcopy "${elf_path}" -O binary "${bin_path}"
cp "${bin_path}" .
