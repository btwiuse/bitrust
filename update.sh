#! /bin/bash

BASE=`pwd`
GH_PAGES="${BASE}/.git/gh-pages/"

has_changes () {
  ! git diff --quiet
}

date;
git submodule update --remote rust \
&& bitrust > "${GH_PAGES}log.json" \
&& cd $GH_PAGES \
&& has_changes \
&& git commit -a -m "Manual update" \
&& git push \
|| echo "No changes";

