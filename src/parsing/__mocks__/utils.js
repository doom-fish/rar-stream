// @flow
import { mockStreamFromString } from './mock-buffer-stream';

export function newPadding(count: number) {
    return Array(count * 2).fill('0').join('');
}

export function hammingWeight(num: number) {
    num = num - (num >> 1 & 0x55555555);
    num = (num & 0x33333333) + (num >> 2 & 0x33333333);
    return (num + (num >> 4) & 0xf0f0f0f) * 0x1010101 >> 24;
}

export default (Parser: any, size: number) => ({
    newParser(binaryStr: string) {
        return new Parser(mockStreamFromString(binaryStr, { size: size }));
    },
    parseHeader(field: string, binaryStr: string) {
        return new Parser(
            mockStreamFromString(binaryStr, { size: size })
        ).parse()[field];
    }
});

export function btoh(binary: any) {
    const str = binary.toString(16);
    return str.length % 2 !== 0 ? '0' + str : str;
}
