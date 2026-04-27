import assert from "node:assert/strict";
import {
  buildExportFilename,
  rowsToCsv,
  serialiseExport,
} from "./dashboardExport";

interface TestCase {
  name: string;
  run: () => void | Promise<void>;
}

export const tests: TestCase[] = [
  {
    name: "rowsToCsv returns empty string for empty input",
    run: () => {
      assert.equal(rowsToCsv([]), "");
    },
  },
  {
    name: "rowsToCsv emits header row plus values",
    run: () => {
      const csv = rowsToCsv([
        { id: "loan-1", amount: 1000 },
        { id: "loan-2", amount: 250 },
      ]);
      assert.equal(csv, "id,amount\r\nloan-1,1000\r\nloan-2,250");
    },
  },
  {
    name: "rowsToCsv quotes values containing delimiter or quotes",
    run: () => {
      const csv = rowsToCsv([
        { name: "Alice, Bob", note: 'has "quotes"', empty: null },
      ]);
      assert.equal(csv, 'name,note,empty\r\n"Alice, Bob","has ""quotes""",');
    },
  },
  {
    name: "rowsToCsv unions keys across heterogeneous rows",
    run: () => {
      const csv = rowsToCsv([{ a: 1 }, { b: 2 }]);
      assert.equal(csv, "a,b\r\n1,\r\n,2");
    },
  },
  {
    name: "rowsToCsv prefixes formula-trigger values to mitigate CSV injection",
    run: () => {
      const csv = rowsToCsv([
        { formula: "=SUM(A1+A2)" },
        { formula: "+1+1" },
        { formula: "-2-2" },
        { formula: "@cmd" },
        { formula: "\tinjected" },
        { formula: "safe value" },
      ]);
      assert.equal(
        csv,
        [
          "formula",
          "'=SUM(A1+A2)",
          "'+1+1",
          "'-2-2",
          "'@cmd",
          "'\tinjected",
          "safe value",
        ].join("\r\n"),
      );
    },
  },
  {
    name: "rowsToCsv combines formula prefix with quoting for embedded delimiters",
    run: () => {
      const csv = rowsToCsv([{ formula: "=A1,B2" }]);
      assert.equal(csv, 'formula\r\n"\'=A1,B2"');
    },
  },
  {
    name: "serialiseExport returns indented JSON",
    run: () => {
      const json = serialiseExport([{ id: "loan-1" }], "json");
      assert.equal(json, '[\n  {\n    "id": "loan-1"\n  }\n]');
    },
  },
  {
    name: "buildExportFilename slugifies and stamps the filename",
    run: () => {
      const filename = buildExportFilename("Stello Vault!! Report", "csv");
      assert.match(filename, /^stello-vault-report-\d{4}-\d{2}-\d{2}T.*\.csv$/);
    },
  },
];
