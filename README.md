# glory-tunnel
A minimal 1-on-1 terminal messenger cli written purely in rust.

## Description
This is an extremely minimalistic version of any sort of messenger piece of software, except it's only meant for 1 on 1 sessions and everything takes place in a terminal.

## Purpose
This was just a fun little experiment messing with the `console` crate as well as rust's ability with concurrency and networking.

## Hosting
`./glory-tunnel --host <ADDR:PORT> <USERNAME> <PASSWORD>`

## Connecting
`./glory-tunnel --connect <ADDR:PORT> <USERNAME> <PASSWORD>`

**USERNAME** doesn't have to be the same to join the other person. It is only an identifier.

## Notes
I've only run this on a select few amount of different terminals. The development of this project took place in VSCode where I used the integrated terminal for all of my testings. This terminal works with glory-tunnel's terminal manipulation perfectly fine but the Windows command prompt doesn't do too hot. I honestly didn't spend too much time on this project so there may be a few blocking issues here and there, but for the most part, if run in a compatible terminal, it runs great and extremely smooth utilizing little to zero resources.
