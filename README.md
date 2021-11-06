# url-shortener

A URL shortener written in Rust.

### Extra notes
ROCKET_PORT in the .env can be used to set the server's port
ROCKET_ADDRESS in the .env can be used to set the server's address.
See more [here](https://docs.rs/rocket/0.4.10/rocket/config/)

## Setup on Linux:
```
# Fill in your .env.example
nano .env.example

# Move the filled in .env.example to .env
mv .env.example .env

# Run the code using
cargo run
# or run the compiled binary using
./url-shortener
```


## Setup on Windows
- Fill in your .env.example
- rename it to .env
- run the server using `cargo run` or if you downloaded a binary run the `url-shortener.exe` file.


## Setup on Mac:
I have no idea, feel free to add one with a pull request

## Notes
> You can also run this on docker using the provided Dockerfile.