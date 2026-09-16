/** Turns a Specta command result into the value React Query expects. */
export async function unwrapCommandResult<T>(
  result: Promise<
    | { status: "ok"; data: T }
    | { status: "error"; error: string }
  >,
): Promise<T> {
  const response = await result;
  if (response.status === "error") throw new Error(response.error);
  return response.data;
}
