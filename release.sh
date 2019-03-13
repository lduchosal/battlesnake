#!bin/sh -x

# sh release.sh 0.11 albertfish

VERSION=$1
CODENAME=$2
DIR=/home/battlesnake

RELEASE=v$VERSION-$CODENAME
FOLDER=battlesnake-$VERSION-$CODENAME
URL=https://github.com/lduchosal/battlesnake/archive/$RELEASE.tar.gz

cd $DIR
. .profile
fetch $URL
tar -xvzf $RELEASE.tar.gz
cd $FOLDER
nohup cargo run --release &

sleep 5
tail nohup.out
