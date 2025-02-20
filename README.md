# picoauth

> 🤝 | smol opinionated headless authn server

because holy $h\*t other solutions are overkill for my small projects. but also, i wanted to know how hard is it to **properly** handle these details;

this project aims to keep everything simple / minimal while compromising little to none in terms of security. read more about the motivation of this project below.

## highlights

**authentication only** -- picoauth only does authn; you are free to implement authorization / permission systems on your own which _may_ ease your development cycle

**actually fully headless** -- you are the one who "connects" a login page. you are the ones who decide to use any features.

**fully permissive by default** -- designed to do the bare-minimum (but secure) authn; you are the ones who need to handle client-side captcha / rate-limiting / etc.

**first-party username / password login method** -- users are not forced to register using an email (although encouraged for forget password); you may take control of this and force client-side

**(soon) support for 2fa (via totp & passkey)** -- 'nuff said

**stateless-first** -- jwt first to minimize auth-server dependency

## features:

- [x] http api
    - [x] http over tcp socket
    - [x] http over unix socket
- [ ] authn
    - [x] jwt
        - [x] access token generation
        - [x] refresh token generation
        - [x] re-refresh access token
        - [x] access token validation
        - [x] refresh token validation
        - [ ] invalidate all previous jwt (semi-hacky?)
    - [ ] cookie
        - [ ] in-memory redis alternative?
        - [ ] refresh cookie
- [x] password login
    - [x] registration
    - [x] login
    - [x] password reset
        - [ ] limit password reset interval - prevents harrassment & SMTP spam
    - [ ] account update (display name, email)
    - [ ] account deletion
    - [ ] hibp checking
- [ ] admin api
    - [ ] users GET / POST / DELETE
    - [ ] user GET / PUT / DELETE
- [ ] multi-factor authentication
    - [ ] registration
    - [ ] deletion
    - [ ] time-based one-time passwords
    - [ ] webauthn login
        - [ ] passkey login
        - [ ] FIDO U2F login
- [ ] email verification
- [ ] oauth2 (probably never lol)

## storage method:

- libsql (sqlite)
    - db encryption available
- hashed + salted password
    - extra algo secret padding available
- jwt
    - HS256, looking to update to EC512
- email
    - direct smtp
    - some kind of universal mq to let other server handle emails

## stateless-first

- ‼️ Session provider
    - JWT is stateless
    - Cookie requires some kind of KV store, could be in-memory or redis or other rust alternative
- ‼️ Storage provider
    - libsql only, other non-embedded db solutions are simply too overkill for my purposes
- ❗ Notification provider
    - email is stateless, requires SMTP config that's set-and-forget

## motivation

- **I want to try to build this as a production-ready experiment** -- They say that "auth is a timesink which is full of bombs", welp, i'm looking forward to finding out if that is true.
- I prefer a classic username / password login
- Every app's authorization requirements & approach is inherently different -- It might be better to let them handle it themselves & focus on authn only
- JWT is preferred as it minimizes database lookup. Verify over lookups.
- Security is as strongest as its weakest link, i want to benchmark the weakest link of a self-hosted auth
- Looking fwd to stress test these kind of apps
- Strive to be stateless - Maybe having an _authn_ option where serving "stale" data may be valid?
- Other solutions are overkill
- Other solutions takes too much resource (ram + storage + processing power) (looking at you keycloak & jvm)
- Other solutions takes too much time to setup
- Other solutions might provide tons of unused features which takes resources
