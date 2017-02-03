// @flow
import { mockStreamFromString } from './mock-buffer-stream';
import AbstractParser from '../abstract-parser';

export function newPadding(count: number): string {
    return Array(count * 2).fill('0').join('');
}

export function hammingWeight(num: number): number {
    num = num - (num >> 1 & 0x55555555);
    num = (num & 0x33333333) + (num >> 2 & 0x33333333);
    return (num + (num >> 4) & 0xf0f0f0f) * 0x1010101 >> 24;
}

export default (Parser: typeof AbstractParser, size: number) => ({
    newParser(binaryStr: string): AbstractParser {
        return new Parser(mockStreamFromString(binaryStr, { size: size }));
    },
    parseHeader(field: string, binaryStr: string): number {
        return new Parser(
            mockStreamFromString(binaryStr, { size: size })
        ).parse()[field];
    }
});

export function btoh(binary: number): string {
    const str = binary.toString(16);
    return str.length % 2 !== 0 ? '0' + str : str;
}
