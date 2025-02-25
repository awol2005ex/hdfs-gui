import { invoke } from "@tauri-apps/api/core";


export const get_avro_content = async (
  id: number,
  filePath: string
) => {
  const result: Array<Object> = await invoke("get_avro_content", {
    id: id,
    filePath: filePath,
  });
  return result;
};