# Building VDR Proxy image locally


## Dependencies

### Certificates
If you're on a machine provisioned by ATB, you will likely encounter SSL issues either through VPN or through Netskope. Please note that you can run Oliu locally outside of our Pulse VPN. However, since we can't disable Netskope, please ensure you download the following cert `ca-certificates.crt`:
```
https://drive.google.com/file/d/17XeZY0jaCo34TnIn1yylKLGRnqx_bfsU/view
```

### Building
1. Clone the vdr-proxy repo
```bash
git clone git@github.com:atb-leap/indy-vdr.git
```

2. `cd indy-vdr`
3. Copy over the `ca-certificates.crt` file in the root of the project

4. Build the image with the right tag
```bash
docker build . -t gcr.io/leap-0123/didaas/didaas-vdr:latest
```