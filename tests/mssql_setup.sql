IF OBJECT_ID('Employees','U') IS NOT NULL DROP TABLE Employees;
IF OBJECT_ID('Countries','U') IS NOT NULL DROP TABLE Countries;
CREATE TABLE Countries (CountryId NVARCHAR(3) PRIMARY KEY, Name NVARCHAR(50));
CREATE TABLE Employees (EmployeeId INT PRIMARY KEY, FirstName NVARCHAR(50), CountryId NVARCHAR(3), HireDate DATETIME);
INSERT INTO Countries (CountryId, Name) VALUES ('Mex','Mexico'), ('USA','United States');
INSERT INTO Employees (EmployeeId, FirstName, CountryId, HireDate) VALUES (1, 'Luis', 'Mex', GETDATE());
GO
