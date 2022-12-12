export const bufferFromString = (str, size, variant = "hex") => {
    if (size) {
        let padding = Math.abs(+size - str.length / 2);
        str += Array(padding).fill("00").join("");
    }
    return Buffer.from(str, variant);
};
export const bind = (Parser) => ({
    parseHeader(field, binaryStr) {
        const p = new Parser(bufferFromString(binaryStr, Parser.HEADER_SIZE));
        const parsed = p.parse();
        // @ts-ignore
        return parsed[field];
    },
});
export const newPadding = (count) => Array(count * 2)
    .fill("0")
    .join("");
export const hammingWeight = (num) => {
    num = num - ((num >> 1) & 0x55555555);
    num = (num & 0x33333333) + ((num >> 2) & 0x33333333);
    return (((num + (num >> 4)) & 0xf0f0f0f) * 0x1010101) >> 24;
};
export const btoh = (binary) => {
    const str = binary.toString(16);
    return str.length % 2 !== 0 ? "0" + str : str;
};
