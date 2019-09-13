## Why Rust now

For one: the Python version was always meant as a quick prototype. Python isn’t stable
software. 

Secondly:

Python:
`xargs -a 100-quoted.list ./librarian.py read  9.14s user 0.09s system 99% cpu 9.250 total`

```
-------------------------------------------------------------------------
 Language      Files        Lines         Code     Comments       Blanks
-------------------------------------------------------------------------
 Python            9          236          174            1           61
-------------------------------------------------------------------------
 Total             9          236          174            1           61
-------------------------------------------------------------------------
```

Rust:
`cargo run -- -vv read -f 100.list  5.14s user 0.51s system 98% cpu 5.735 total`

```
-------------------------------------------------------------------------
 Language      Files        Lines         Code     Comments       Blanks
-------------------------------------------------------------------------
 Rust              2          132          106            3           23
-------------------------------------------------------------------------
 Total             2          132          106            3           23
-------------------------------------------------------------------------
```

Both are very naive implementation but the Rust one is twice as fast, less lines
of code and does more already (e.g. EPUB)
