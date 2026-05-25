mkdir -p -f /atmosphere/contents/01007EF00011E000/exefs
cd /atmosphere/contents/01007EF00011E000/exefs
mput -e target/megaton/none/test-mod/test-mod.nso
mput -e target/megaton/none/test-mod/main.npdm
rm -f subsdk9
mv test-mod.nso subsdk9
