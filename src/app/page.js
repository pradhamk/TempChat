import { Button, Link } from "@nextui-org/react";

export default function Home() {
  return (
    <main className="h-screen flex justify-center items-center flex-col">
      <h1 className="text-5xl font-bold mb-8">TempChat</h1>
      <Button color="primary" variant="ghost" className="mt-4 mb-4 pr-8 pl-8 pb-6 pt-6" as={Link} href="/handle?type=create">
        <h1 className="text-xl">Create</h1>
      </Button>
      <Button color="primary" variant="ghost" className="mt-4 mb-4 pr-8 pl-8 pb-6 pt-6" as={Link} href="/handle?type=join">
        <h1 className="text-xl">Join</h1>
      </Button>
    </main>
  );
}
