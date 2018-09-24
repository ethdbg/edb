#!/bin/bash
mkfifo utf8out

< /dev/urandom perl -CO -ne '
    BEGIN{$/=\3}
    no warnings "utf8";
    $c = unpack("L>","\0$_") * 0x10f800 >> 24;
    $c += 0x800 if $c >= 0xd800;
    $c = chr $c;
    print $c if $c =~ /\P{unassigned}/' > utf8out &

dd if=utf8out of=small_file.txt bs=1048576 count=10 # create a 99MB file of utf8

rm utf8out


