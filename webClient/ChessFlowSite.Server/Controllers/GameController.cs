using ChessFlowSite.Server.Models;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using ReactApp1.Server.Data;
using System.ComponentModel.DataAnnotations.Schema;

namespace ChessFlowSite.Server.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    public class GameController : Controller
    {
        private readonly ApplicationDbContext _db;

        public GameController(ApplicationDbContext db)
        {
            _db = db;
        }
        [HttpGet("show/{gameID}")]
        [Authorize]
        public IActionResult Show(int gameID)
        {
            var game = _db.Games
            .Include(g => g.PlayerWhite)
            .Include(g => g.PlayerBlack)
            .FirstOrDefault(g => g.Id == gameID);
            if (game == null && game?.Result != "InProgress")
            {
                return NotFound(new { errors = new[] { new { code = "GameNotFound", description = "Game not found" } } });
            }
            return Ok(new GameDataModel(game));
        }
        [HttpGet("index")]
        [Authorize]
        public async Task<IActionResult> Index([FromQuery] GameIndexModel model)
        {
            if (model.Page <= 0 || model.PageSize <= 0)
                return BadRequest(new { errors = new[] { new { code = "InvalidPagination", description = "Page and pageSize must be greater than zero." } } });

            var query = _db.Games
                .Include(g => g.PlayerWhite)
                .Include(g => g.PlayerBlack)
                .AsQueryable();

            //filter games that are in progress
            query = query.Where(g => g.Result != "InProgress");

            //filter guest-guest or guest-bot games
            query = query.Where(g => (g.PlayerWhiteId != null || g.PlayerBlackId != null));

            var u1 = model.UsernameOne;
            var u2 = model.UsernameTwo;

            // Filtering
            if (!string.IsNullOrWhiteSpace(u1) && !string.IsNullOrWhiteSpace(u2))
            {
                query = query.Where(g => (((g.GuestWhiteName ?? g.PlayerWhite.Name) == u1) && ((g.GuestBlackName ?? g.PlayerBlack.Name) == u2))
                || (((g.GuestWhiteName ?? g.PlayerWhite.Name) == u2) && ((g.GuestBlackName ?? g.PlayerBlack.Name) == u1)));
            }
            else if (!string.IsNullOrEmpty(u1))
            {
                query = query.Where(g => (g.GuestWhiteName ?? g.PlayerWhite.Name) == u1 || (g.GuestBlackName ?? g.PlayerBlack.Name) == u1);
            }
            else if (!string.IsNullOrEmpty(u2)) {
                query = query.Where(g => (g.GuestWhiteName ?? g.PlayerWhite.Name) == u2 || (g.GuestBlackName ?? g.PlayerBlack.Name) == u2);
            }


            // Sorting
            query = (model.SortType.ToLower(), model.IsAscending) switch
            {
                ("date", true) => query.OrderBy(r => r.StartTime),
                ("date", false) => query.OrderByDescending(r => r.StartTime),
                ("elodif", true) => query.OrderBy(r => Math.Abs(r.DeltaEloWhite ?? 0 - r.DeltaEloWhite ?? 0)),
                ("elodif", false) => query.OrderByDescending(r => Math.Abs(r.DeltaEloWhite ?? 0 - r.DeltaEloWhite ?? 0)),
                ("id", true) => query.OrderBy(r => r.Id),
                _ => query.OrderByDescending(r => r.Id), // Default
            };

            // Pagination
            var totalItems = await query.CountAsync();
            var lastPage = Math.Ceiling((double)totalItems / model.PageSize);
            var games = await query
                .Skip((model.Page - 1) * model.PageSize)
                .Take(model.PageSize)
                .ToListAsync();

            var result = new
            {
                lastPage = lastPage,
                items = games.Select(r => new GameCarouselModel(r))
            };

            return Ok(result);
        }
    }

    public class GameDataModel
    {
        public string WhiteName { get; set; }
        public string BlackName { get; set; }
        public int WhiteElo { get; set; }
        public int BlackElo { get; set; }

        public int WhiteDeltaElo { get; set; }
        public int BlackDeltaElo { get; set; }

        public bool IsWhiteGuest { get; set; }
        public bool IsBlackGuest { get; set; }

        public bool IsWhiteReportable { get; set; }
        public bool IsBlackReportable { get; set; }
        public bool IsBotGame { get; set; }
        public string Format { get; set; } // "Blitz", "Bullet", "Classical"
        public string Result { get; set; }
        public string Fen { get; set; }
        public string PGN { get; set; }

        public DateTime StartTime { get; set; }

        public GameDataModel(Game game)
        {
            WhiteName = game.GuestWhiteName ?? game.PlayerWhite?.Name;
            BlackName = game.GuestBlackName ?? game.PlayerBlack?.Name;
            WhiteElo = game.EloWhite;
            BlackElo = game.EloBlack;
            WhiteDeltaElo = game.DeltaEloWhite ?? 0;
            BlackDeltaElo = game.DeltaEloBlack ?? 0;
            IsWhiteGuest = game.GuestWhiteName != null;
            IsBlackGuest = game.GuestBlackName != null;
            IsBotGame = game.IsBotGame;
            IsWhiteReportable = game.PlayerWhite != null && game.PlayerWhite.isBanned == false;
            IsBlackReportable = game.PlayerBlack != null && game.PlayerBlack.isBanned == false;
            Format = game.Format ?? "Classical"; // Default to Classical if not set
            Result = game.Result;
            Fen = game.FinalFEN;
            PGN = game.PGN;
            StartTime = (DateTime)game.StartTime;
        }
    }

    public class GameCarouselModel
    {
        public GameDataModel Game { get; set; }
        public int GameId { get; set; }
        public GameCarouselModel(Game game)
        {
            Game = new GameDataModel(game);
            GameId = game.Id;
        }
    }

    public class GameIndexModel {
        public string? UsernameOne { get; set; }
        public string? UsernameTwo { get; set; }
        public int Page { get; set; } = 1;
        public int PageSize { get; set; } = 10;
        public string? SortType { get; set; }
        public bool IsAscending { get; set; } = false;
    }
}
