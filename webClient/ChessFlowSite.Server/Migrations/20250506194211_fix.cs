using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ChessFlowSite.Server.Migrations
{
    /// <inheritdoc />
    public partial class fix : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "FK_Reports_AspNetUsers_ReporterId",
                table: "Reports");

            migrationBuilder.RenameColumn(
                name: "ReporterId",
                table: "Reports",
                newName: "ReportedId");

            migrationBuilder.RenameIndex(
                name: "IX_Reports_ReporterId",
                table: "Reports",
                newName: "IX_Reports_ReportedId");

            migrationBuilder.AddForeignKey(
                name: "FK_Reports_AspNetUsers_ReportedId",
                table: "Reports",
                column: "ReportedId",
                principalTable: "AspNetUsers",
                principalColumn: "Id",
                onDelete: ReferentialAction.Restrict);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "FK_Reports_AspNetUsers_ReportedId",
                table: "Reports");

            migrationBuilder.RenameColumn(
                name: "ReportedId",
                table: "Reports",
                newName: "ReporterId");

            migrationBuilder.RenameIndex(
                name: "IX_Reports_ReportedId",
                table: "Reports",
                newName: "IX_Reports_ReporterId");

            migrationBuilder.AddForeignKey(
                name: "FK_Reports_AspNetUsers_ReporterId",
                table: "Reports",
                column: "ReporterId",
                principalTable: "AspNetUsers",
                principalColumn: "Id",
                onDelete: ReferentialAction.Restrict);
        }
    }
}
