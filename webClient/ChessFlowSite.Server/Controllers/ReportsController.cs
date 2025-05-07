using ChessFlowSite.Server.Models;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using ReactApp1.Server.Data;

namespace ChessFlowSite.Server.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    public class ReportsController : Controller
    {

        private readonly ApplicationDbContext _db;
        private readonly UserManager<ApplicationUser> _userManager;

        public ReportsController(ApplicationDbContext db, UserManager<ApplicationUser> userManager)
        {
            _db = db;
            _userManager = userManager;
        }
        [HttpPost("create")]
        [Authorize]
        public async Task<IActionResult> Create([FromBody] ReportModel model)
        {
            var reported =  _db.ApplicationUsers.FirstOrDefault(u => u.Name == model.ReportedName);
            var reportee = await _userManager.GetUserAsync(User);

            if (reportee.Name != model.ReporteeName || reported == null || reported == reportee) {
                return BadRequest(new { errors = new[] { new { code = "InvalidUsers", description = "Reported and/or reportee users are invalid" } } });
            }

            if (model.Reason.Length > 511) {
                ModelState.AddModelError("ContentTooLong", "Content cannot be longer than 511 characters.");
            }
            if (model.Reason.Length < 1) {
                ModelState.AddModelError("ContentTooShort", "Reason must be at least 1 character");
            }
            if (!ModelState.IsValid) {
                var errorList = ModelState.Where(ms => ms.Value.Errors.Count > 0).SelectMany(kvp => kvp.Value.Errors.Select(e => new
                    {
                        code = kvp.Key,
                        description = e.ErrorMessage
                    })).ToArray();
                return BadRequest(new { errors = errorList });
            }

                Report report = new Report();
            report.Reported = reported;
            report.Reportee = reportee;
            if (model.GameId != null) {
                var game = _db.Games.FirstOrDefault(g => g.Id == model.GameId);
                if (game != null) {
                    report.Game = game;
                }
            }
            report.Reason = model.Reason;
            _db.Reports.Add(report);
            await _db.SaveChangesAsync();
            return Ok();
        }
        [HttpGet("show")]
        [Authorize(Roles = "Admin")]
        public async Task<IActionResult> Show([FromQuery] ShowModel model)
        {
            if (model.Page <= 0 || model.PageSize <= 0)
                return BadRequest(new { errors = new[] { new { code = "InvalidPagination", description = "Page and pageSize must be greater than zero." } } });

            var query = _db.Reports
                .Include(r => r.Reported)
                .Include(r => r.Reportee)
                .Include(r => r.Game)
                .AsQueryable();

            // Filtering
            if (!string.IsNullOrWhiteSpace(model.ReportedName))
            {
                query = query.Where(r => r.Reported.Name == model.ReportedName);
            }

            Console.WriteLine("\n\n!!!!!!!!!!!!\n" + model.SortType + " " + model.IsAscending + "\n!!!!!!!!!!!!\n");

            // Sorting
            query = (model.SortType.ToLower(), model.IsAscending) switch
            {
                ("date", true) => query.OrderBy(r => r.Created),
                ("date", false) => query.OrderByDescending(r => r.Created),
                ("id", true) => query.OrderBy(r => r.Id),
                _ => query.OrderByDescending(r => r.Id), // Default
            };

            // Pagination
            var totalItems = await query.CountAsync();
            var lastPage = Math.Ceiling((double)totalItems / model.PageSize);
            var reports = await query
                .Skip((model.Page - 1) * model.PageSize)
                .Take(model.PageSize)
                .ToListAsync();

            var result = new
            {
                lastPage = lastPage,
                items = reports.Select(r => new {
                    reportID = r.Id,
                    reportedName = r.Reported?.Name,
                    reporteeName  = r.Reportee?.Name,
                    gameID = r.GameId,
                    reason = r.Reason,
                    created = r.Created
                })
            };

            return Ok(result);
        }
    }
}
public class ReportModel
{
    public string ReportedName{ get; set; }
    public string ReporteeName{ get; set; }
    public int? GameId { get; set; }
    public string Reason { get; set; }
}

public class ShowModel
{
    public string? ReportedName { get; set; }
    public int Page { get; set; } = 1;
    public int PageSize { get; set; } = 10;
    public string? SortType { get; set; }
    public bool IsAscending { get; set; } = false;
}
