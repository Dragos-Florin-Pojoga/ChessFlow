using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ChessFlowSite.Server.Migrations
{
    /// <inheritdoc />
    public partial class RolesAndElo : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<int>(
                name: "Elo",
                table: "AspNetUsers",
                type: "INTEGER",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "Elo",
                table: "AspNetUsers");
        }
    }
}
