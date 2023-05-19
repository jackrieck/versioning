# versioning

A simple poc to demonstrate a way to version Solana Accounts

# how to

```bash
# start anchor localnet without deploying program
anchor localnet --skip-deploy

# deploy the program
anchor build && anchor deploy --provider.cluster http://localhost:8899 --program-name versioning --program-keypair target/deploy/versioning-keypair.json

# run the test to setup the account as version 1
anchor build && anchor test --skip-deploy --skip-local-validator

# uncomment all the 'bar/baz' code, skip the initialize test and unskip the migrate test

# upgrade the program
anchor build && anchor upgrade --provider.cluster http://localhost:8899 --program-id 8EEY7nX8xNTgNHvsYAhKZ8TwLP6eJLEhBZGbJWx3vtD6 target/deploy/versioning.so

# run the migrate script to upgrade the data account to version 2
anchor build && anchor test --skip-deploy --skip-local-validator
```