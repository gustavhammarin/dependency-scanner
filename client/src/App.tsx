import { Outlet } from "react-router-dom";
import NavBar from "./components/NavBar";
import "./index.css";

function App() {
  return (
    <div className="min-h-screen bg-background">
      <div className="mx-auto flex min-h-screen w-full">
        <NavBar />
        <main className="flex-1 overflow-x-hidden px-4 pb-6 pt-20 sm:px-6 md:px-8 md:pt-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}

export default App;
