using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ChessFlowSite.Server.Migrations
{
    /// <inheritdoc />
    public partial class fixGame : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<int>(
                name: "DeltaEloBlack",
                table: "Games",
                type: "INTEGER",
                nullable: true);

            migrationBuilder.AddColumn<int>(
                name: "DeltaEloWhite",
                table: "Games",
                type: "INTEGER",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "DeltaEloBlack",
                table: "Games");

            migrationBuilder.DropColumn(
                name: "DeltaEloWhite",
                table: "Games");
        }
    }
}
