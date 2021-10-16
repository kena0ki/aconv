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
iconv -t aschii  utf8_en.txt     > aschii_en.txt

# MEMO
# encoding_rs does not seem to handle simbols in sjis properly
# e.g.
#  '〜'(\x81\x60) in SJIS is mapped to \xEF\xBD\x9E in UTF8, although expected to \xE3\x80\x9C in UTF8
#  '−'(\x81\x7c) in SJIS is mapped to \xEF\xBC\x8D in UTF8, although expected to \xE2\x88\x92 in UTF8
# So I exclude the simbols from utf8_ja.txt in the mean time encoding_rs is fixed.

