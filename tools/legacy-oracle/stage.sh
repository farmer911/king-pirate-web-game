#!/bin/bash
# Copy /home/pirate from a read-only mount of the legacy VM disk into a clean build context.
#
# The VM image is infected (botnet in /bin, /lib and /etc/init.d, extra uid-0 accounts, PHP webshells),
# so only /home/pirate is copied, never an OS file, and the known webshells are left out.
# Nothing from the image is executed here.
#
# Usage: stage.sh <mounted VM root, e.g. /root/kp/mnt> <output dir, e.g. /root/kp/oracle>
set -euo pipefail
SRC="${1:?mounted VM root}/home/pirate"
OUT="${2:?output dir}"
HERE="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$OUT/pirate"
rsync -a --delete --no-specials --no-devices \
  --exclude='/www/T9NuoYa/' --exclude='/www/phpMyAdmin/' \
  --exclude='/www/app/sys/res.php' --exclude='/rpcfw/lib/api/hzw_api.php' \
  --exclude='/lcserver/log/*' --exclude='/dataproxy/log/*' --exclude='/phpproxy/log/*' \
  --exclude='/battle/log/*' --exclude='/rpcfw/log/*' --exclude='/programs/nginx/logs/*' \
  --exclude='/programs/mysql/var/mysql-bin.*' --exclude='/programs/mysql/var/*.err' \
  --exclude='/programs/mysql/var/slow.log' --exclude='/programs/mysql/var/mysql.pid' \
  --exclude='/.bash_history' --exclude='/rpcfw/data/coveragedump/' --exclude='/rpcfw/data/eatmp/' \
  "$SRC/" "$OUT/pirate/"
if grep -rlIE 'web-shell|shellname=' "$OUT/pirate/www" "$OUT/pirate/rpcfw/lib" "$OUT/pirate/rpcfw/conf" "$OUT/pirate/rpcfw/module"; then
  echo "webshell found in the staged copy; remove it before building" >&2
  exit 1
fi
cp "$HERE/Dockerfile" "$HERE/start.sh" "$OUT/"
echo "staged $(du -sh "$OUT/pirate" | cut -f1) in $OUT"
