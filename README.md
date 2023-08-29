[![Validate](https://github.com/haribo256/home-dns-refresh/actions/workflows/validate.yml/badge.svg)](https://github.com/haribo256/home-dns-refresh/actions/workflows/validate.yml)
[![Release](https://github.com/haribo256/home-dns-refresh/actions/workflows/release.yml/badge.svg)](https://github.com/haribo256/home-dns-refresh/actions/workflows/release.yml)

# Home DNS Refresh

Looks up the current external IP, and updates an Azure Zone A Record to that IP if its different.

The A Record must be resolvable, so that it may check if its the same or different.

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
