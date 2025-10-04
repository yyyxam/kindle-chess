#!/bin/sh


echo "Copying KUAL via scp"
# echo "Transferring binaries"
# scp ./kindle_chess/target/... -J root@kindle:/mnt/us/hellokindle
echo "Transferring KUAL-extension"
scp -r ./kindle_KUAL/extensions/* root@kindle:/mnt/us/extensions/

echo "Building Binaries"
cd kindle_chess

# scp -r ./kindle
