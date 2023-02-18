# idea

- given the [beginning url](https://blog.acolyer.org/2014/10/08/a-storm-drain-for-the-morning-paper/) of morning paper determine all following morning paper articles
- find the next one by get the html file and using the href with `rel="next"`

## TODO

- [ ] add dates to generated json
- [x] generate RSS feed
  - note `mp_url/feed` returns the comments not the RSS feed
- [ ] update RSS feed on a daily basis (using Github actions maybe?)
  - can validate RSS using [Feed validation service](https://validator.w3.org/feed/check.cgi)
  - how to allow anyone else to start from the beginning of Morning Paper?

## issues

- one issue is that if you miss an article then by the next day the XML file will be updated to a new one and you won't be able to see it

## sanity check

- [first 50 papers](https://blog.acolyer.org/2014/10/15/themorningpaper-reaches-50-papers/)
- since only starts at paper 45 on blog will have to manually add from paper 1 till paper 44
  - 30th July 2014, “Why Functional Programming Matters,” John Hughes, 1990

## perf notes

### don't create client and just call directly

```rust
let content = reqwest::get(search_for_url.clone()).await?.text().await?;
```

```
real    91.42s
user    13.88s
sys     1.73s
```

### having client outside loop (5x times speed up)

```rust
let content = client
  .get(search_for_url.clone())
  .send()
  .await?
  .text()
  .await?;
```

```
real    16.47s
user    6.64s
sys     0.76s
```
