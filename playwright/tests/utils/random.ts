export function randomName(): string {
  const characters = "abcdefghijklmnopqrstuvwxyz";

  let result = "";
  for (let i = 0; i < 5; i++) {
    result += characters.charAt(Math.floor(Math.random() * characters.length));

    if (i === 0) {
      result = result.toUpperCase();
    }
  }

  return result;
}
