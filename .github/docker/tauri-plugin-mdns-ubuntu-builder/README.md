# tauri-plugin-mdns Ubuntu Builder Docker Image

## Updating the Docker image

> [!NOTE]
> This is usually not needed to do manually: the `docker-reusable.yml`
> workflow in `hrzlgnm/actions` builds the image and pushes it to GHCR
> whenever the `Dockerfile` changes (`v1` and `latest` from `main`,
> `dev` and `dev-latest` from other branches).

To publish an image manually (e.g. the first `v1`, or a new `v2` after
a breaking base change), run the following commands:

    $ docker build --tag ghcr.io/hrzlgnm/tauri-plugin-mdns-ubuntu-builder:VERSION_NUMBER_HERE .github/docker/tauri-plugin-mdns-ubuntu-builder/
    $ docker login ghcr.io -u YOUR_GITHUB_USER_NAME_HERE
    $ docker push ghcr.io/hrzlgnm/tauri-plugin-mdns-ubuntu-builder:VERSION_NUMBER_HERE

Then, point the `container:` pins in the `.github/workflows/`
directory at the new tag.
