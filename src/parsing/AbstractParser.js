import { Parser } from 'binary-parser';

export default class AbstractParser {
  parse(){
    throw Error("Abstract Method, implement in sub classes");
  }
}