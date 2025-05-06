using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ChessFlowSite.Server.Migrations
{
    /// <inheritdoc />
    public partial class DatabaseStructure : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AlterColumn<int>(
                name: "Elo",
                table: "AspNetUsers",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0,
                oldClrType: typeof(int),
                oldType: "INTEGER",
                oldNullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "isBanned",
                table: "AspNetUsers",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.CreateTable(
                name: "Games",
                columns: table => new
                {
                    Id = table.Column<int>(type: "INTEGER", nullable: false)
                        .Annotation("Sqlite:Autoincrement", true),
                    PlayerWhiteId = table.Column<string>(type: "TEXT", nullable: true),
                    PlayerBlackId = table.Column<string>(type: "TEXT", nullable: true),
                    GuestWhiteName = table.Column<string>(type: "TEXT", nullable: true),
                    GuestBlackName = table.Column<string>(type: "TEXT", nullable: true),
                    BotId = table.Column<string>(type: "TEXT", nullable: true),
                    IsBotGame = table.Column<bool>(type: "INTEGER", nullable: false),
                    IsRated = table.Column<bool>(type: "INTEGER", nullable: false),
                    Result = table.Column<string>(type: "TEXT", nullable: false),
                    EloWhite = table.Column<int>(type: "INTEGER", nullable: true),
                    EloBlack = table.Column<int>(type: "INTEGER", nullable: true),
                    MoveCount = table.Column<int>(type: "INTEGER", nullable: false),
                    Format = table.Column<string>(type: "TEXT", nullable: true),
                    StartTime = table.Column<DateTime>(type: "TEXT", nullable: true),
                    EndTime = table.Column<DateTime>(type: "TEXT", nullable: true),
                    FinalFEN = table.Column<string>(type: "TEXT", nullable: true),
                    PGN = table.Column<string>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_Games", x => x.Id);
                    table.ForeignKey(
                        name: "FK_Games_AspNetUsers_PlayerBlackId",
                        column: x => x.PlayerBlackId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Restrict);
                    table.ForeignKey(
                        name: "FK_Games_AspNetUsers_PlayerWhiteId",
                        column: x => x.PlayerWhiteId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Restrict);
                });

            migrationBuilder.CreateTable(
                name: "Reports",
                columns: table => new
                {
                    Id = table.Column<int>(type: "INTEGER", nullable: false)
                        .Annotation("Sqlite:Autoincrement", true),
                    ReporterId = table.Column<string>(type: "TEXT", nullable: false),
                    ReporteeId = table.Column<string>(type: "TEXT", nullable: false),
                    GameId = table.Column<int>(type: "INTEGER", nullable: true),
                    Reason = table.Column<string>(type: "TEXT", maxLength: 511, nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_Reports", x => x.Id);
                    table.ForeignKey(
                        name: "FK_Reports_AspNetUsers_ReporteeId",
                        column: x => x.ReporteeId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Restrict);
                    table.ForeignKey(
                        name: "FK_Reports_AspNetUsers_ReporterId",
                        column: x => x.ReporterId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Restrict);
                    table.ForeignKey(
                        name: "FK_Reports_Games_GameId",
                        column: x => x.GameId,
                        principalTable: "Games",
                        principalColumn: "Id");
                });

            migrationBuilder.CreateTable(
                name: "Bans",
                columns: table => new
                {
                    Id = table.Column<int>(type: "INTEGER", nullable: false)
                        .Annotation("Sqlite:Autoincrement", true),
                    BannedId = table.Column<string>(type: "TEXT", nullable: false),
                    IssuerId = table.Column<string>(type: "TEXT", nullable: false),
                    Reason = table.Column<string>(type: "TEXT", maxLength: 511, nullable: false),
                    ReportId = table.Column<int>(type: "INTEGER", nullable: true),
                    Permanent = table.Column<bool>(type: "INTEGER", nullable: false),
                    BannedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    BannedUntil = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_Bans", x => x.Id);
                    table.ForeignKey(
                        name: "FK_Bans_AspNetUsers_BannedId",
                        column: x => x.BannedId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Restrict);
                    table.ForeignKey(
                        name: "FK_Bans_AspNetUsers_IssuerId",
                        column: x => x.IssuerId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Restrict);
                    table.ForeignKey(
                        name: "FK_Bans_Reports_ReportId",
                        column: x => x.ReportId,
                        principalTable: "Reports",
                        principalColumn: "Id");
                });

            migrationBuilder.CreateIndex(
                name: "IX_AspNetUsers_Name",
                table: "AspNetUsers",
                column: "Name",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "IX_Bans_BannedId",
                table: "Bans",
                column: "BannedId");

            migrationBuilder.CreateIndex(
                name: "IX_Bans_IssuerId",
                table: "Bans",
                column: "IssuerId");

            migrationBuilder.CreateIndex(
                name: "IX_Bans_ReportId",
                table: "Bans",
                column: "ReportId");

            migrationBuilder.CreateIndex(
                name: "IX_Games_PlayerBlackId",
                table: "Games",
                column: "PlayerBlackId");

            migrationBuilder.CreateIndex(
                name: "IX_Games_PlayerWhiteId",
                table: "Games",
                column: "PlayerWhiteId");

            migrationBuilder.CreateIndex(
                name: "IX_Reports_GameId",
                table: "Reports",
                column: "GameId");

            migrationBuilder.CreateIndex(
                name: "IX_Reports_ReporteeId",
                table: "Reports",
                column: "ReporteeId");

            migrationBuilder.CreateIndex(
                name: "IX_Reports_ReporterId",
                table: "Reports",
                column: "ReporterId");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "Bans");

            migrationBuilder.DropTable(
                name: "Reports");

            migrationBuilder.DropTable(
                name: "Games");

            migrationBuilder.DropIndex(
                name: "IX_AspNetUsers_Name",
                table: "AspNetUsers");

            migrationBuilder.DropColumn(
                name: "isBanned",
                table: "AspNetUsers");

            migrationBuilder.AlterColumn<int>(
                name: "Elo",
                table: "AspNetUsers",
                type: "INTEGER",
                nullable: true,
                oldClrType: typeof(int),
                oldType: "INTEGER");
        }
    }
}
