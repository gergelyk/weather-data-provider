# Weather Data Aggregator

Agregates data from meteo stations.

Application is available at: https://weather.krason.dev/

Supported providers:
- https://www.aemet.es
- https://www.meteo.cat
- https://www.meteoclimatic.net


## Development

```sh
set-env SPIN_VARIABLE_API_TOKEN demo
spin up --build
curl -X POST -d @api/examples/mixed.json 'http://127.0.0.1:3000/api/v1?token=demo'
```

Notes:
- Use `spin watch` to rebuild & run the app on changes.
- Temporarily comment out line with `data-wasm-opt="z"` in `index.html`
  and section [profile.release] in Cargo.toml for faster development cycle.

## Deployment

```sh
set-env SPIN_VARIABLE_API_TOKEN (tr -dc A-Za-z0-9 </dev/urandom | head -c 16)
spin deploy --build --variable api_token=$E:SPIN_VARIABLE_API_TOKEN
curl -X POST -d @api/examples/mixed.json 'https://weather.fermyon.app/api/v1?token='$E:SPIN_VARIABLE_API_TOKEN
```

Notes:
- Temporarily change name of the application in `spin.toml` if you would like to test before deploying to production.

