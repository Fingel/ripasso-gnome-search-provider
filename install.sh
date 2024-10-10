#!/usr/bin/env bash
set -eu -o pipefail
cd "$(dirname "$(realpath "${0}")")"

DATADIR=${DATADIR:-/usr/share}
LIBDIR=${LIBDIR:-/usr/lib}

install -Dm 0755 target/release/ripasso-gnome-search-provider "${LIBDIR}"/ripasso-gnome-search-provider/ripasso-gnome-search-provider

install -Dm 0644 conf/io.m51.Pass.search-provider.ini "${DATADIR}"/gnome-shell/search-providers/io.m51.Pass.search-provider.ini

install -Dm 0644 conf/io.m51.Pass.SearchProvider.desktop "${DATADIR}"/applications/io.m51.Pass.SearchProvider.desktop

install -Dm 0644 conf/io.m51.Pass.SearchProvider.service.dbus "${DATADIR}"/dbus-1/services/io.m51.Pass.SearchProvider.service

install -Dm 0644 conf/io.m51.Pass.SearchProvider.service.systemd "${LIBDIR}"/systemd/user/io.m51.Pass.SearchProvider.service
