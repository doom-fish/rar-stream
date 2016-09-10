import { mockStreamFromString } from './mock-buffer-stream';

export function newPadding(count) {
	return Array(count * 2).fill('0').join('');
}

export function hammingWeight(num) {
	num = num - ((num >> 1) & 0x55555555);
	num = (num & 0x33333333) + ((num >> 2) & 0x33333333);
	return ((num + (num >> 4) & 0xF0F0F0F) * 0x1010101) >> 24;
}

export default (Parser, size) => ({
		newParser(binaryStr) {
			return new Parser(mockStreamFromString(binaryStr, { size: size }));
		},
		parseHeader(field, binaryStr) {
			return  new Parser(mockStreamFromString(binaryStr, { size: size })).parse()[field];
		}
});

export function btoh(binary) {
	const str = binary.toString(16);
	return str.length % 2 !== 0 ? '0' + str : str;
}
