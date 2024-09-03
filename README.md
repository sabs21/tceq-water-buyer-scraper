# TCEQ Scraper
A CLI tool that quickly fetches and stores water buyer relationship data from the TCEQ website.

## Getting Started
To get started, open up the TCEQ Scraper release folder in command line, then run ".\tceq-scraper -h" without quotes. This is a CLI tool, so you'll need to interact with tceq-scraper.exe this VIA command line. 

This release includes everything you need to get going, including:
- The scraper itself.
- SQLite database which the scraper uses to store each water detail and relationship.
- Examples to follow for how to format the input CSV.

Note: The output argument is currently non-functional. All output is stored within water_buyer_relationships.db3.

WARNING: **DO NOT RENAME "water_buyer_relationships.db3" OR ELSE THE SCRAPER WILL NOT WORK!** 

Tool for accessing, interacting with, and exporting the database:
- [SQLite Studio Download](https://sqlitestudio.pl/)
