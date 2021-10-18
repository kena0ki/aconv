// There is no evidence these appear more frequently than NON_TEXTS. It's just my guess.
pub const NON_TEXTS_FREQUENT: [char; 2] =  [
    '\u{0000}', // Null character	NUL
    '\u{FFFD}', // REPLACEMENT CHARACTER
];
// These characters are based on the file command.
// https://github.com/file/file/blob/ac3fb1f582ea35c274ad776f26e57785c4cf976f/src/encoding.c#L236
pub const NON_TEXTS: [char; 24] =  [
    '\u{0001}', // Start of Heading	SOH
    '\u{0002}', // Start of Text	STX
    '\u{0003}', // End-of-text character	ETX
    '\u{0004}', // End-of-transmission character	EOT
    '\u{0005}', // Enquiry character	ENQ
    '\u{0006}', // Acknowledge character	ACK
    // '\u{0007}', // Bell character	BEL
    // '\u{0008}', // Backspace	BS
    // '\u{0009}', // Horizontal tab	HT
    // '\u{000A}', // Line feed	LF
    // '\u{000B}', // Vertical tab	VT
    // '\u{000C}', // Form feed	FF
    // '\u{000D}', // Carriage return	CR
    '\u{000E}', // Shift Out	SO
    '\u{000F}', // Shift In	SI
    '\u{0010}', // Data Link Escape	DLE
    '\u{0011}', // Device Control 1	DC1
    '\u{0012}', // Device Control 2	DC2
    '\u{0013}', // Device Control 3	DC3
    '\u{0014}', // Device Control 4	DC4
    '\u{0015}', // Negative-acknowledge character	NAK
    '\u{0016}', // Synchronous Idle	SYN
    '\u{0017}', // End of Transmission Block	ETB
    '\u{0018}', // Cancel character	CAN
    '\u{0019}', // End of Medium	EM
    '\u{001A}', // Substitute character	SUB
    // '\u{001B}', // Escape character	ESC
    '\u{001C}', // File Separator	FS
    '\u{001D}', // Group Separator	GS
    '\u{001E}', // Record Separator	RS
    '\u{001F}', // Unit Separator	US
    // ...
    '\u{007F}', // Delete	DEL
];

