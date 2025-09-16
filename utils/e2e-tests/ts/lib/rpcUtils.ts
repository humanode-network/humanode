//! Common RPC utils.

interface JsonRpcResponse {
  result?: any;
  error?: {
    code: number;
    message: string;
  };
}

export async function customRpcRequest(
  endpoint: string,
  method: string,
  params: any[] = [],
): Promise<any> {
  const data = {
    jsonrpc: "2.0",
    id: 1,
    method,
    params,
  };

  const response = await fetch(endpoint, {
    method: "POST",
    body: JSON.stringify(data),
    headers: { "Content-Type": "application/json" },
  });

  const responseData = (await response.json()) as JsonRpcResponse;

  if (responseData.error) {
    throw new Error(responseData.error.message);
  }

  return responseData.result;
}
