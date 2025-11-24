import { tableFromIPC } from 'apache-arrow';
import { ArrowParseError } from '../error.js';

export class ArrowReader {
  parse(arrayBuffer, fileName = 'unknown') {
    try {
      return tableFromIPC(arrayBuffer);
    } catch (error) {
      throw new ArrowParseError(fileName, error);
    }
  }

  tableToRows(table) {
    const rows = [];
    for (const row of table) {
      rows.push({ ...row });
    }
    return rows;
  }
}
