# Cargoal

## Test

To run the tests, run `cargo test` in the root directory of this project.

Before running the tests, you need to run the `Dockerfile` in the cargoal directory. To do this, run the following command:

```bash
docker build -t cargoal .
```

After running the `Dockerfile`, you can run the tests with the following command:

```bash
docker run -d -p 5432:5432 cargoal
``` 