#!/bin/bash
# Start the legacy stack in the order its operators used (from /home/pirate/.bash_history in the VM),
# then follow the game logs. Runs as the original `pirate` user, except nginx (binds :80).
set -u
H=/home/pirate
# setsid: CentOS 7 `su` kills the processes left in its session when the command returns.
as_pirate() { setsid su pirate -s /bin/bash -c "$1" </dev/null; }

# Use this container's address instead of the VM's old one (192.168.1.234): lcserver's public listener
# (lcserver/conf/game20002.xml:44), the platform API URLs (rpcfw/conf/PlatformApi.cfg.php:22-27), the IP
# whitelist (rpcfw/conf/WhiteList.cfg.php:24) and the GM API scripts (rpcfw/lib/api/hzw_*.php).
OLD_IP=$(cat $H/.bound_ip 2>/dev/null || echo 192.168.1.234)
NEW_IP=$(hostname -i | awk '{print $1}')
if [ "$OLD_IP" != "$NEW_IP" ]; then
  grep -rlF "$OLD_IP" $H/lcserver/conf $H/rpcfw/conf $H/rpcfw/lib/api | xargs -r sed -i "s/${OLD_IP//./\\.}/$NEW_IP/g"
  echo "$NEW_IP" > $H/.bound_ip
fi

# Fewer PHP workers than the VM's 50 (programs/php/etc/php-fpm.conf:161-176): php-fpm's master holds
# about 1 GB of static game data (btstore) shared copy-on-write, and every busy worker slowly copies it.
sed -i -e 's/^pm.max_children = .*/pm.max_children = 8/' -e 's/^pm.start_servers = .*/pm.start_servers = 2/' \
       -e 's/^pm.min_spare_servers = .*/pm.min_spare_servers = 1/' -e 's/^pm.max_spare_servers = .*/pm.max_spare_servers = 3/' \
       $H/programs/php/etc/php-fpm.conf

mkdir -p $H/lcserver/log $H/dataproxy/log $H/phpproxy/log $H/battle/log $H/rpcfw/log $H/programs/nginx/logs
chown -R pirate:pirate $H/lcserver/log $H/dataproxy/log $H/phpproxy/log $H/battle/log $H/rpcfw/log

# Same flags as programs/memcached/run.sh, distro binary (see Dockerfile).
memcached -d -u pirate -l 127.0.0.1 -m 2048
as_pirate "cd $H/programs/mysql && bin/mysqld_safe --defaults-file=$H/programs/mysql/etc/my.cnf >/dev/null 2>&1 &"
for _ in $(seq 60); do [ -S $H/programs/mysql/var/mysql.sock ] && break; sleep 1; done
# The image ships this MyISAM table marked as crashed; MySQL's automatic repair fails, mysqlcheck works.
# Credentials from dataproxy/conf/config.xml:85-88.
$H/programs/mysql/bin/mysqlcheck -upirate -padmin --socket=$H/programs/mysql/var/mysql.sock \
  --repair pirate20002 t_battle_record_7 >/dev/null
as_pirate "$H/programs/php/sbin/php-fpm"
(cd $H/programs/nginx && sbin/nginx)
# Each <module>/run.sh starts `bin/supervisor.<module> bin/<module> <args>`. The supervisor (a restarter)
# segfaults on this kernel, so the daemons start directly with the same arguments (none for dataproxy,
# phpproxy and battle; `-c conf/game20002.xml` for lcserver, from lcserver/run.sh).
# Ruffle can stall the client for minutes while it loads a module and miss lcserver's 60 s heartbeat
# (lcserver/conf/game20002.xml:28; it closes after 120 s of silence). The oracle allows 10 minutes.
sed -i 's|<heartbeat_interval>60</heartbeat_interval>|<heartbeat_interval>600</heartbeat_interval>|' \
  $H/lcserver/conf/game20002.xml
for m in dataproxy phpproxy battle; do as_pirate "cd $H/$m && bin/$m >log/supervisor.log 2>&1 &"; sleep 1; done
as_pirate "cd $H/lcserver && bin/lcserver -c conf/game20002.xml >log/game20002.supervisor.log 2>&1 &"

sleep 3
netstat -ltnp
exec tail -F $H/lcserver/log/*.log $H/rpcfw/log/*.log $H/phpproxy/log/*.log $H/dataproxy/log/*.log $H/battle/log/*.log 2>/dev/null
