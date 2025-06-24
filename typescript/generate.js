#!/usr/bin/env node

// import { execFileSync } from "child_process"
import { compile } from 'json-schema-to-typescript'
import fs from 'fs';

// execFileSync("cargo", ["run"])

const jsonSchema = JSON.parse(fs.readFileSync("./schema.json", "utf8"));
const clientMethods = JSON.parse(fs.readFileSync("./target/client_requests.json", "utf8"));
const agentMethods = JSON.parse(fs.readFileSync("./target/agent_requests.json", "utf8"));

let typescriptSource = await compile(jsonSchema, "Agent Coding Protocol", {
  additionalProperties: false,
  bannerComment: false,
});

const clientInterface = requestMapToInterface("Client", clientMethods);
const agentInterface = requestMapToInterface("Agent", agentMethods);

typescriptSource += '\n' + clientInterface + '\n\n' + agentInterface + '\n';

fs.writeFileSync("typescript/schema.ts", typescriptSource, 'utf8')

function requestMapToInterface(name, methods) {
  let code = `export interface ${name} {\n`;

  for (const { name, request_type, response_type } of methods) {
    const jsMethodName = toJsMethodName(name)
    code += `  ${jsMethodName}(params: ${request_type}): Promise<${response_type}>;\n`;
  }
  code += '}\n\n';

  code += `export const ${name.toUpperCase()}_METHODS = {`
  code += '\n'
  for (const { name } of methods) {
    const jsMethodName = toJsMethodName(name)
    code += `  "${name}": "${jsMethodName}",`;
    code += '\n';
  }
  code += '};';

  return code;
}

function toJsMethodName(name) {
  const words = name.split("_");
  return words.map((word, index) => {
    if (index == 0) {
      return word
    } else {
      return word[0].toUpperCase() + word.slice(1)
    }
  }).join('')
}
