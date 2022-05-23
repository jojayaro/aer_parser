# Parser for Alberta Energy Regulator ST1 Report written in Rust (WIP)

Parser was originally written in Python [Repo Link](https://github.com/jojayaro/Exploration_App) but I decided to refactor it to Rust as part of learning process and to improve the code since the original implementation is not optimal.

Note that this parser only extracts the data for the licences issued. I will eventually include a parser for the whole file to also include the amendments and cancellations.

Since it is written in Rust the idea is to write to Delta Lake files using Delta-rs but it is still a work in progress.