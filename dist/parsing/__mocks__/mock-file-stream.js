import { Readable } from "stream";
export class MockFileStream extends Readable {
    object;
    options;
    constructor(object, options) {
        super(options);
        this.object = object;
        this.options = options;
    }
    _read() {
        if (!!this.object &&
            typeof this.options.start === "number" &&
            typeof this.options.end === "number") {
            const buffer = this.object.slice(this.options.start, this.options.end);
            this.push(buffer);
            this.object = null;
        }
        else {
            this.push(this.object);
            this.object = null;
        }
    }
}
