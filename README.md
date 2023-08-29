# Home DNS Refresh

[![Crate](https://img.shields.io/crates/v/home-dns-refresh.svg)](https://crates.io/crates/home-dns-refresh)
[![Validate](https://github.com/haribo256/home-dns-refresh/actions/workflows/validate.yml/badge.svg)](https://github.com/haribo256/home-dns-refresh/actions/workflows/validate.yml)
[![Release](https://github.com/haribo256/home-dns-refresh/actions/workflows/release.yml/badge.svg)](https://github.com/haribo256/home-dns-refresh/actions/workflows/release.yml)

`home-dns-refresh` is a CLI job that looks up the current external IP, and updates an Azure Zone A Record to that IP if its different.

The A Record must be resolvable, so that it may check if its the same or different.

## Installing locally

```sh
cargo install home-dns-refresh
```

## How to run the command

Run the command to display the usage:

  ```sh
  home-dns-refresh --help
  ```

Which will show the options to run with:

  ```
  home-dns-refresh
  Updates an Azure DNS zone A record to the currently external IP address

  USAGE:
      home-dns-refresh [OPTIONS] --client-id <client-id> --client-secret <client-secret> --resource-group <resource-group-name> --sub-domain <sub-domain> --subscription-id <subscription-id> --tenant-id <tenant-id> --zone-name <zone-name>

  FLAGS:
      -h, --help       Prints help information
      -V, --version    Prints version information

  OPTIONS:
      -c, --client-id <client-id>                   
      -p, --client-secret <client-secret>           
      -r, --resource-group <resource-group-name>    
      -d, --sub-domain <sub-domain>                 
      -s, --subscription-id <subscription-id>       
      -t, --tenant-id <tenant-id>                   
          --ttl <ttl>                               
      -z, --zone-name <zone-name>   
  ```

## Setup on Nomad as a periodic job

```hcl
job "home_dns_refresh" {
  datacenters = ["dc1"]
  type = "batch"

  periodic {
    cron                = "*/5 * * * * *"
    prohibit_overlap    = false
  }

  group "home_dns_refresh" {
    count = 1
    task "home_dns_refresh" {
      driver = "docker"

      config {
        image = "ghcr.io/haribo256/haribo256/home-dns-refresh:<version>"

        args = [
          "--tenant-id", "<tenant-id>",
          "--subscription-id", "<subscription-id>",
          "--resource-group", "<resouce-group>",
          "--zone-name", "<azure-zone>",
          "--sub-domain", "<subdomain-on-azure-zone-to-update>",
          "--client-id", "<azure-client-id>",
          "--client-secret", "<azure-client-secret>"
        ]
      }

      restart {
        attempts = 0
        mode = fail
      }

      resources {
        cpu    = 100
        memory = 20
      }
    }
  }
}
```
