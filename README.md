[![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/docker-container.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/powered-by-energy-drinks.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/60-percent-of-the-time-works-every-time.svg)](https://forthebadge.com)
# PasteIt📋 - Backend

Backend for [PasteIt](https://paste.051205.xyz) - a lightweight, self-hosted pastebin-like 
service built with Rust for fast and secure text sharing.

## Features
- Syntax highlighting
- Rate limiting by ip
  - Note: IPs are saved in the database using SHA256 hashing for added privacy
- Database paste storage
  - Note: Paste content is encrypted using AES-256
- reCAPTCHA protected upload
- OOTB logic for docker secrets, .env files and system environment

Planned:
- Expiry time for pastes
- Password protected pastes
- API support for raw data fetching (Already partially functional)

## Installation
Dependencies:
- rustc 1.84+ (Might work with older versions, untested)
- Postgres 17 (Older versions may be compatible)
- All the crates in [Cargo.toml](Cargo.toml)

## Usage
>[!Note]
> You can visit /api/docs for all the available endpoints

Types are indicated using `Rust type | Typescript type` \
Creating new pastes:
- Call `POST /api/pastes/paste` which accepts these arguments
  - `name: Option<String> | string`
  - `paste: String | string`
  - `language: String | string`
  - `recaptchaToken: String | string`

Getting pastes:
- Call `GET /api/pastes/paste/{short_id}` which will return this in its content
  - `name: String | string`
  - `paste: String | string`
  - `language: String | string`
  - `createdAt: DateTime<UTC> | Date`

> [!Note]
> This project also has a [WebUI counterpart](https://github.com/milkyicedtea/PasteIt-Frontend)
> which follows the same licensing

## Configuration
Necessary environment variables for correct functioning:
  - DB_URL - URL string to connect to your database of choice 
  (possibly postgres for best compatibility)
  - PASTE_ENCRYPTION_KEY - Random hex string that can be generated with `openssl rand -hex 32`
  or any equivalent
  - RECAPTCHA_SECRET_KEY - reCAPTCHA key you can get from registering on recaptcha
>[!Note]
> The current default port is 8080 and can be changed [here](src/main.rs#L94) if needed.
> Might use a secret for the port in future versions

## Deployment
Although this can simply be ran using `cargo run --release` (for production),
the included [Dockerfile](Dockerfile) provides a pretty nice docker image for running this 
in a container, and it doesn't take long at all! \
Just use `docker build -t <container-name> .` to build it. \
Then deploy it using one of these two options:
- Docker service
```bash
docker service create --name pasteit --secret DB_URL --secret PASTE_ENCRYPTION_KEY --secret RECAPTCHA_SECRET_KEY --publish 8080:8080 pasteit-backend
```
- Docker stack
```bash
docker stack deploy -c docker-compose.yml --detach=false pasteit
```

I recommend checking these two StackOverflow threads to better understand the difference between the two.\
[1](https://stackoverflow.com/questions/41833622/docker-create-service-vs-docker-deploy-stack)\
[2](https://stackoverflow.com/questions/44329083/what-is-the-difference-between-docker-service-and-stack)\

TDLR, both work equally good for this :)

## Issues and contributing
Opening issues and contributing is always welcome within the [license's](LICENSE.md) permissions

## Third-Party Licenses
This project uses third-party libraries that are licensed under MIT and/or Apache 2.0.
These libraries are used as-is, without modification. Please refer to their respective
repositories for more details on their licenses.
