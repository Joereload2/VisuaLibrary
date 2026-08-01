import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StationPlaceholder } from "./StationPlaceholder";

describe("StationPlaceholder", () => {
  it("renders title, purpose and auxiliary notice", () => {
    render(
      <StationPlaceholder
        title="Library"
        purpose="Consultar recursos aprobados."
        notes={["Solo approved"]}
      />,
    );

    expect(screen.getByRole("heading", { name: "Library" })).toBeInTheDocument();
    expect(screen.getByText("Consultar recursos aprobados.")).toBeInTheDocument();
    expect(screen.getByText(/Vista auxiliar/i)).toBeInTheDocument();
    expect(screen.getByText("Solo approved")).toBeInTheDocument();
  });
});
