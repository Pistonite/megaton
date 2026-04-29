mkdir -p -f /atmosphere/contents/01007EF00011E000/exefs
cd /atmosphere/contents/01007EF00011E000/exefs
mput -e target/megaton/none/example.nso
mput -e target/megaton/none/main.npdm
rm -f subsdk9
mv example.nso subsdk9
