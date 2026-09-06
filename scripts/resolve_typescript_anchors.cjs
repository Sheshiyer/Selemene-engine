#!/usr/bin/env node
"use strict";

const path = require("node:path");

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(2);
}

let ts;
try {
  ts = require("typescript");
} catch {
  fail("TypeScript compiler API is unavailable");
}

let request;
try {
  const chunks = [];
  process.stdin.setEncoding("utf8");
  process.stdin.on("data", (chunk) => chunks.push(chunk));
  process.stdin.on("end", () => {
    try {
      request = JSON.parse(chunks.join(""));
    } catch {
      fail("anchor parser request is not valid JSON");
    }
    resolveAnchors(request);
  });
} catch {
  fail("anchor parser could not read its request");
}

function resolveAnchors(value) {
  if (
    value === null ||
    typeof value !== "object" ||
    typeof value.fileName !== "string" ||
    typeof value.source !== "string"
  ) {
    fail("anchor parser request must contain fileName and source strings");
  }

  const extension = path.extname(value.fileName).toLowerCase();
  const scriptKinds = new Map([
    [".ts", ts.ScriptKind.TS],
    [".mts", ts.ScriptKind.TS],
    [".cts", ts.ScriptKind.TS],
    [".js", ts.ScriptKind.JS],
    [".mjs", ts.ScriptKind.JS],
    [".cjs", ts.ScriptKind.JS],
  ]);
  const scriptKind = scriptKinds.get(extension);
  if (scriptKind === undefined) {
    fail(`unsupported TypeScript parser extension: ${extension || "<none>"}`);
  }

  const sourceFile = ts.createSourceFile(
    value.fileName,
    value.source,
    ts.ScriptTarget.Latest,
    true,
    scriptKind,
  );
  if (sourceFile.parseDiagnostics.length > 0) {
    const diagnostic = sourceFile.parseDiagnostics[0];
    const message = ts.flattenDiagnosticMessageText(diagnostic.messageText, " ");
    let location = "";
    if (typeof diagnostic.start === "number") {
      const position = sourceFile.getLineAndCharacterOfPosition(diagnostic.start);
      location = `:${position.line + 1}:${position.character + 1}`;
    }
    fail(`TypeScript parse diagnostic ${value.fileName}${location}: ${message}`);
  }

  const anchors = new Set();

  function addAnchor(qualifiers, name) {
    if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(name)) {
      anchors.add([...qualifiers, name].join("::"));
    }
  }

  function identifierName(node) {
    return node && node.name && ts.isIdentifier(node.name) ? node.name.text : null;
  }

  function collectBindingName(name, qualifiers) {
    if (ts.isIdentifier(name)) {
      addAnchor(qualifiers, name.text);
      return;
    }
    for (const element of name.elements) {
      if (ts.isBindingElement(element)) {
        collectBindingName(element.name, qualifiers);
      }
    }
  }

  function collectClassLikeMembers(node, qualifiers) {
    for (const member of node.members) {
      const name = identifierName(member);
      if (name !== null) {
        addAnchor(qualifiers, name);
      }
    }
  }

  function collectModuleBody(body, qualifiers) {
    if (body === undefined) {
      return;
    }
    if (ts.isModuleBlock(body)) {
      for (const statement of body.statements) {
        collectStatement(statement, qualifiers);
      }
      return;
    }
    if (ts.isModuleDeclaration(body)) {
      collectStatement(body, qualifiers);
    }
  }

  function collectStatement(statement, qualifiers) {
    if (ts.isVariableStatement(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        collectBindingName(declaration.name, qualifiers);
      }
      return;
    }

    const name = identifierName(statement);
    if (name === null) {
      return;
    }

    addAnchor(qualifiers, name);
    const ownedQualifiers = [...qualifiers, name];
    if (ts.isClassDeclaration(statement) || ts.isInterfaceDeclaration(statement)) {
      collectClassLikeMembers(statement, ownedQualifiers);
    } else if (ts.isModuleDeclaration(statement)) {
      collectModuleBody(statement.body, ownedQualifiers);
    }
  }

  for (const statement of sourceFile.statements) {
    collectStatement(statement, []);
  }

  process.stdout.write(
    `${JSON.stringify({ anchors: [...anchors].sort() })}\n`,
  );
}
