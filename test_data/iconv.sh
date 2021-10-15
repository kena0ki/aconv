echo -en "\xFF\xFE" > utf16le_BOM_th.txt && iconv -t utf-16le      utf8_th.txt     >> utf16le_BOM_th.txt
echo -en "\xFE\xFF" > utf16be_BOM_th.txt && iconv -t utf-16be      utf8_th.txt     >> utf16be_BOM_th.txt
iconv -t utf-16le      utf8_th.txt     > utf16le_th.txt
iconv -t utf-16be      utf8_th.txt     > utf16be_th.txt
iconv -t sjis          utf8_ja.txt     > sjis_ja.txt
iconv -t euc-jp        utf8_ja.txt     > euc-jp_ja.txt
iconv -t iso-2022-jp   utf8_ja.txt     > iso-2022-jp_ja.txt
iconv -t big5          utf8_zh_CHT.txt > big5_zh_CHT.txt
iconv -t gbk           utf8_zh_CHS.txt > gbk_zh_CHS.txt
iconv -t gb18030       utf8_zh_CHS.txt > gb18030_zh_CHS.txt
iconv -t euc-kr        utf8_ko.txt     > euc-kr_ko.txt
iconv -t koi8-r        utf8_ru.txt     > koi8-r_ru.txt
iconv -t windows-1252  utf8_es.txt     > windows-1252_es.txt

echo -en "\xFE\xFF" > utf16be_ja.txt && iconv -t utf-16be      utf8_ja.txt     >> utf16be_ja.txt
iconv -t utf-16be      utf8_ja.txt     > utf16be_ja.txt

