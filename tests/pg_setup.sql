DROP TABLE IF EXISTS Employees;
DROP TABLE IF EXISTS Countries;
CREATE TABLE Countries (
    CountryId VARCHAR(3) PRIMARY KEY,
    Name VARCHAR(50) NOT NULL
);
CREATE TABLE Employees (
    EmployeeId INT PRIMARY KEY,
    FirstName VARCHAR(50),
    CountryId VARCHAR(3) REFERENCES Countries(CountryId),
    HireDate TIMESTAMP
);
INSERT INTO Countries (CountryId, Name) VALUES ('Mex','Mexico'), ('USA','United States');
INSERT INTO Employees (EmployeeId, FirstName, CountryId, HireDate) VALUES
    (1,'Luis','Mex','2023-01-01 00:00:00'),
    (2,'Ana','Mex','2024-01-01 00:00:00'),
    (3,'John','USA','2022-01-01 00:00:00');
